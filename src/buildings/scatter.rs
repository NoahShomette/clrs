use crate::buildings::{get_neighbors_tilepos, tile_cost_check, Activate, Building, TileNode, check_is_colorable};
use crate::color_system::{convert_tile, ColorConflictEvent, TileColor};
use bevy::prelude::{
    Commands, Component, Entity, EventWriter, FromReflect, Query, Reflect, With, Without,
};
use bevy::utils::hashbrown::HashMap;
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage, TilemapSize};
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::{ObjectGridPosition, ObjectId};
use bevy_ggf::player::PlayerMarker;
use rand::Rng;

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct Scatters {
    pub scatter_range: u32,
    pub scatter_amount: u32,
}

pub fn simulate_scatterers(
    mut tile_storage_query: Query<(Entity, &MapId, &TileStorage, &TilemapSize)>,
    pulsers: Query<
        (
            Entity,
            &ObjectId,
            &PlayerMarker,
            &Building<Scatters>,
            &ObjectGridPosition,
        ),
        (Without<MapId>, With<Activate>),
    >,
    mut tiles: Query<
        (
            Entity,
            &TileTerrainInfo,
            Option<(&mut PlayerMarker, &mut TileColor)>,
        ),
        (With<Tile>, Without<Building<Scatters>>, Without<MapId>),
    >,
    mut event_writer: EventWriter<ColorConflictEvent>,
    mut commands: Commands,
) {
    let Some((_, _, tile_storage, tilemap_size)) = tile_storage_query
        .iter_mut()
        .find(|(_, id, _, _)| id == &&MapId{ id: 1 })else{
        return;
    };

    for (entity, id, player_marker, scatter, object_grid_position) in pulsers.iter() {
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
                    scatter.building_type.scatter_range,
                    &neighbor.0,
                    &current_node.tile_pos,
                    &mut tiles_info,
                ) {
                    continue 'neighbors;
                }

                unvisited_tiles.push(*tiles_info.get_mut(&neighbor.0).expect(
                    "Is safe because we know we add the node in at the beginning of this loop",
                ));
            }

            unvisited_tiles.remove(0);
            visited_nodes.push(current_node.tile_pos);
        }

        let mut rng = rand::thread_rng();

        for _ in 0..=scatter.building_type.scatter_amount {
            let y: usize = rng.gen_range(0..visited_nodes.len());

            let Some(tile_entity) = tile_storage.get(&visited_nodes[y]) else {
                continue;
            };
            if let Ok((entity, tile_terrain_info, options)) = tiles.get_mut(tile_entity) {
                if !check_is_colorable(tile_terrain_info) {
                    continue;
                }
                if convert_tile(
                    id,
                    &player_marker.id(),
                    visited_nodes[y],
                    tile_terrain_info,
                    &options,
                    &mut event_writer,
                ) {}
            }
        }

        commands.entity(entity).remove::<Activate>();
    }
}
