use crate::buildings::{Building, Pulser};
use crate::color_system::{convert_tile, ColorConflictEvent, TileColor, TileColorStrength};
use bevy::prelude::{Commands, Entity, EventWriter, Mut, Query, With, Without};
use bevy::utils::hashbrown::HashMap;
use bevy::utils::petgraph::visit::Walker;
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage, TilemapSize};
use bevy_ggf::game_core::state::Changed;
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::{ObjectGridPosition, ObjectId};
use bevy_ggf::player::PlayerMarker;

#[derive(Default, Clone, Copy, Eq, Hash, Debug, PartialEq)]
pub struct TileNode {
    cost: Option<u32>,
    prior_node: TilePos,
    tile_pos: TilePos,
}

// two parts - we pulse outwards, checking the outside neighbors of each tile. If the outside neighbors
// are not the same player then we damage their color by one. Otherwise at that point we stop.
pub fn simulate_pulsers(
    mut tile_storage_query: Query<(Entity, &MapId, &TileStorage, &TilemapSize)>,
    pulsers: Query<
        (
            Entity,
            &ObjectId,
            &PlayerMarker,
            &Building<Pulser>,
            &ObjectGridPosition,
        ),
        Without<MapId>,
    >,
    mut tiles: Query<
        (
            Entity,
            &TileTerrainInfo,
            Option<(&mut PlayerMarker, &mut TileColor)>,
        ),
        (With<Tile>, Without<Building<Pulser>>, Without<MapId>),
    >,
    mut event_writer: EventWriter<ColorConflictEvent>,
) {
    let Some((_, _, tile_storage, tilemap_size)) = tile_storage_query
        .iter_mut()
        .find(|(_, id, _, _)| id == &&MapId{ id: 1 })else{
        return;
        };

    for (entity, id, player_marker, pulser, object_grid_position) in pulsers.iter() {
        let mut tiles_info: HashMap<TilePos, TileNode> = HashMap::new();

        // insert the starting node at the moving objects grid position
        tiles_info.insert(
            object_grid_position.tile_position,
            TileNode {
                tile_pos: object_grid_position.tile_position,
                prior_node: object_grid_position.tile_position,
                cost: Some(0),
            },
        );

        event_writer.send(ColorConflictEvent {
            from_object: *id,
            tile_pos: object_grid_position.tile_position,
            player: player_marker.id(),
        });

        // unvisited nodes
        let mut unvisited_tiles: Vec<TileNode> = vec![TileNode {
            tile_pos: object_grid_position.tile_position,
            prior_node: object_grid_position.tile_position,
            cost: Some(0),
        }];
        let mut visited_nodes: Vec<TilePos> = vec![];

        while !unvisited_tiles.is_empty() {
            unvisited_tiles.sort_by(|x, y| x.cost.unwrap().partial_cmp(&y.cost.unwrap()).unwrap());

            let Some(current_node) = unvisited_tiles.get(0) else {
                continue;
        };

            let neighbor_pos = get_neighbors_tilepos(current_node.tile_pos, &tilemap_size);

            let current_node = *current_node;
            let mut neighbors: Vec<(TilePos, Entity)> = vec![];
            for neighbor in neighbor_pos.iter() {
                let Some(tile_entity) = tile_storage.get(neighbor) else {
                    continue;
                };
                neighbors.push((*neighbor, tile_entity));
            }

            'neighbors: for neighbor in neighbors.iter() {
                if visited_nodes.contains(&neighbor.0) {
                    continue;
                }

                if tiles_info.contains_key(&neighbor.0) {
                } else {
                    let node = TileNode {
                        tile_pos: neighbor.0,
                        prior_node: current_node.tile_pos,
                        cost: None,
                    };
                    tiles_info.insert(neighbor.0, node);
                }

                if !tile_cost_check(
                    pulser.building_type.strength,
                    &neighbor.0,
                    &current_node.tile_pos,
                    &mut tiles_info,
                ) {
                    continue 'neighbors;
                }

                let Some(tile_entity) = tile_storage.get(&neighbor.0) else {
                    continue;
                };
                if let Ok((entity, tile_terrain_info, options)) = tiles.get_mut(tile_entity) {
                    if convert_tile(
                        id,
                        &player_marker.id(),
                        neighbor.0,
                        tile_terrain_info,
                        options,
                        &mut event_writer,
                    ) {
                        unvisited_tiles.push(*tiles_info.get_mut(&neighbor.0).expect(
                            "Is safe because we know we add the node in at the beginning of this loop",
                        ));
                    }
                }
            }

            unvisited_tiles.remove(0);
            visited_nodes.push(current_node.tile_pos);
        }
    }
}

pub fn get_neighbors_tilepos(
    node_to_get_neighbors: TilePos,
    tilemap_size: &TilemapSize,
) -> Vec<TilePos> {
    let mut neighbor_tiles: Vec<TilePos> = vec![];
    let origin_tile = node_to_get_neighbors;
    if let Some(north) =
        TilePos::from_i32_pair(origin_tile.x as i32, origin_tile.y as i32 + 1, tilemap_size)
    {
        neighbor_tiles.push(north);
    }
    if let Some(east) =
        TilePos::from_i32_pair(origin_tile.x as i32 + 1, origin_tile.y as i32, tilemap_size)
    {
        neighbor_tiles.push(east);
    }
    if let Some(south) =
        TilePos::from_i32_pair(origin_tile.x as i32, origin_tile.y as i32 - 1, tilemap_size)
    {
        neighbor_tiles.push(south);
    }
    if let Some(west) =
        TilePos::from_i32_pair(origin_tile.x as i32 - 1, origin_tile.y as i32, tilemap_size)
    {
        neighbor_tiles.push(west);
    }

    neighbor_tiles
}

fn tile_cost_check(
    max_cost: u32,
    tile_pos: &TilePos,
    move_from_tile_pos: &TilePos,
    tile_nodes: &mut HashMap<TilePos, TileNode>,
) -> bool {
    let Some((tile_node, move_from_tile_node)) = get_two_node_mut(tile_nodes, *tile_pos, *move_from_tile_pos) else {
        return false;
    };

    return if tile_node.cost.is_some() {
        if move_from_tile_node.cost.unwrap() + 1 < (tile_node.cost.unwrap()) {
            tile_node.cost = Some(move_from_tile_node.cost.unwrap() + 1);
            tile_node.prior_node = move_from_tile_node.tile_pos;
            true
        } else {
            false
        }
    } else if move_from_tile_node.cost.unwrap() + 1 <= max_cost {
        tile_node.cost = Some(move_from_tile_node.cost.unwrap() + 1);
        tile_node.prior_node = move_from_tile_node.tile_pos;
        true
    } else {
        false
    };
}

/// Returns a mutable reference for both nodes specified and returns them in the same order
fn get_two_node_mut(
    tiles: &mut HashMap<TilePos, TileNode>,
    node_one: TilePos,
    node_two: TilePos,
) -> Option<(&mut TileNode, &mut TileNode)> {
    // either get the current item in the move nodes or make a new default node and add it to the hashmap and then return that
    return if let Some(nodes) = tiles.get_many_mut([&node_one, &node_two]) {
        match nodes {
            [node1, node2] => {
                if node1.tile_pos == node_one {
                    Some((node1, node2))
                } else {
                    Some((node2, node1))
                }
            }
        }
    } else {
        None
    };
}
