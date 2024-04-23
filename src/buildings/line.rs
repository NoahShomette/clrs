use std::collections::BTreeMap;

use crate::buildings::{
    check_is_colorable, get_neighbors_tilepos, tile_cost_check, Activate, Building, TileNode,
};
use crate::color_system::{convert_tile, ColorConflictEvent, TileColor, TileColorStrength};
use crate::objects::{ObjectCachedMap, TileToObjectIndex};
use crate::pathfinding::{AddObjectToTileToObjectIndex, RemoveObjectFromTileToObjectIndex};
use bevy::ecs::world::World;
use bevy::prelude::{
    Commands, Component, Entity, EventWriter, FromReflect, Query, Reflect, With, Without,
};
use bevy::utils::hashbrown::HashMap;
use bevy_ecs_tilemap::map::TilemapSize;
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage};
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::{ObjectGridPosition, ObjectId};
use bevy_ggf::pathfinding::dijkstra::Node;
use bevy_ggf::pathfinding::{MapNode, PathfindMap};
use bevy_ggf::player::PlayerMarker;
use serde::{Deserialize, Serialize};

use super::Simulate;

#[derive(
    Default,
    Clone,
    Eq,
    Hash,
    Debug,
    PartialEq,
    Component,
    Reflect,
    FromReflect,
    Serialize,
    Deserialize,
)]
pub struct Line {
    pub strength: u32,
    pub hits_per_tile: u32,
    pub max_changed_per_side: u32,
}

pub fn simulate_lines_from_cache(
    mut tile_storage_query: Query<(Entity, &MapId, &TileStorage, &TilemapSize)>,
    pulsers: Query<
        (
            Entity,
            &ObjectId,
            &PlayerMarker,
            &Building<Line>,
            &ObjectCachedMap,
            &ObjectGridPosition,
        ),
        (Without<MapId>, With<Activate>, With<Simulate>),
    >,
    mut tiles: Query<
        (
            Entity,
            &TileTerrainInfo,
            Option<(&mut PlayerMarker, &mut TileColor)>,
        ),
        (With<Tile>, Without<Building<Line>>, Without<MapId>),
    >,
    mut event_writer: EventWriter<ColorConflictEvent>,
    mut commands: Commands,
) {
    let Some((_, _, tile_storage, _)) = tile_storage_query
        .iter_mut()
        .find(|(_, id, _, _)| id == &&MapId { id: 1 })
    else {
        return;
    };

    for (entity, id, player_marker, line, cache, object_grid_position) in pulsers.iter() {
        commands.entity(entity).remove::<Activate>();

        let ogp = object_grid_position.tile_position.into();
        let mut tiles_changed: u32 = 0;
        let mut sides_changed = [0u32; 4];

        'main_loop: for tile in cache.cache.iter() {
            let Some(tile_entity) = tile_storage.get(&Into::<TilePos>::into(*tile)) else {
                continue;
            };

            let index = get_side_index(ogp, Into::<TilePos>::into(*tile));
            if sides_changed[index] >= line.building_type.max_changed_per_side {
                continue 'main_loop;
            }
            if let Ok((_, tile_terrain_info, options)) = tiles.get_mut(tile_entity) {
                for _ in 0..line.building_type.hits_per_tile {
                    if let Some((tile_player_marker, tile_color)) = options.as_ref() {
                        if player_marker.id() == tile_player_marker.id() {
                            if let TileColorStrength::Five = tile_color.tile_color_strength {
                            } else {
                                sides_changed[index] += 1;
                                tiles_changed += 1;
                            }
                        } else {
                            sides_changed[index] += 1;
                            tiles_changed += 1;
                        }
                    } else {
                        sides_changed[index] += 1;
                        tiles_changed += 1;
                    }
                    convert_tile(
                        id,
                        &player_marker.id(),
                        Into::<TilePos>::into(*tile),
                        tile_terrain_info,
                        &options,
                        &mut event_writer,
                    );

                    if sides_changed[index] >= line.building_type.max_changed_per_side {
                        continue 'main_loop;
                    }
                }
            }
        }

        if tiles_changed == 0 {
            commands.entity(entity).remove::<Simulate>();
        }
    }
}

fn get_side_index(starting_pos: TilePos, target_pos: TilePos) -> usize {
    if starting_pos.x > target_pos.x {
        return 0;
    }
    if starting_pos.y < target_pos.y {
        return 1;
    }
    if starting_pos.x < target_pos.x {
        return 2;
    }
    3
}

#[derive(Default)]
pub struct LinePathfindMap {
    pub map: HashMap<TilePos, Node>,
}

impl RemoveObjectFromTileToObjectIndex for LinePathfindMap {
    fn remove_from_index(
        &mut self,
        object_id: ObjectId,
        tile_to_object_index: &mut TileToObjectIndex,
    ) {
        for (tile_pos, tile_node) in self.map.iter() {
            if tile_node.valid_move {
                let index_entry = tile_to_object_index
                    .map
                    .entry(tile_pos.clone())
                    .or_default();
                index_entry.retain(|element| element != &object_id);
            }
        }
    }
}

impl AddObjectToTileToObjectIndex for LinePathfindMap {
    fn add_to_index(
        &mut self,
        object_id: ObjectId,
        tile_to_object_index: &mut TileToObjectIndex,
        btree_cache: &mut BTreeMap<u8, Vec<TilePos>>,
    ) {
        for (tile_pos, tile_node) in self.map.iter() {
            if tile_node.valid_move {
                let entry = btree_cache.entry(tile_node.cost() as u8).or_insert(vec![]);
                let index_entry = tile_to_object_index
                    .map
                    .entry(tile_pos.clone())
                    .or_default();
                index_entry.push(object_id);
                entry.push(tile_pos.clone());
            }
        }
    }
}

impl PathfindMap<TilePos, Node, (), Building<Line>> for LinePathfindMap {
    fn new_pathfind_map(&mut self, starting_pos: TilePos) {
        let mut map: HashMap<TilePos, Node> = HashMap::default();
        // insert the starting node at the moving objects grid position
        map.insert(
            starting_pos,
            Node {
                node_pos: starting_pos,
                prior_node_pos: starting_pos,
                move_cost: 0,
                valid_move: true,
                calculated: false,
            },
        );

        self.map = map;
    }

    fn node_cost_calculation(
        &mut self,
        entity_moving: Entity,
        _: Entity,
        tile_pos: TilePos,
        move_from_tile_pos: TilePos,
        world: &World,
    ) -> bool {
        let Some(object_movement) = world.get::<Building<Line>>(entity_moving) else {
            return false;
        };

        let Some([tile_node, move_from_tile_node]) =
            self.map.get_many_mut([&tile_pos, &move_from_tile_pos])
        else {
            return false;
        };

        return if tile_node.calculated {
            if (move_from_tile_node.move_cost + 1) < (tile_node.move_cost) {
                tile_node.move_cost =
                    move_from_tile_node.move_cost + (move_from_tile_node.move_cost + 1);
                tile_node.prior_node_pos = move_from_tile_node.node_pos;
                true
            } else {
                false
            }
        } else if (move_from_tile_node.move_cost + 1)
            <= object_movement.building_type.strength as u32
        {
            tile_node.move_cost = move_from_tile_node.move_cost + 1;
            tile_node.prior_node_pos = move_from_tile_node.node_pos;
            true
        } else {
            false
        };
    }

    fn get_neighbors(&self, node_pos: TilePos, tilemap_size: &TilemapSize) -> Vec<TilePos> {
        let mut neighbor_tiles: Vec<TilePos> = vec![];
        let origin_tile = node_pos;
        let Some(node_to_get_neighbors) = self.get_node(node_pos) else {
            return vec![];
        };
        if node_to_get_neighbors.prior_node_pos == node_to_get_neighbors.node_pos {
            return get_neighbors_tilepos(node_to_get_neighbors.node_pos, tilemap_size);
        }

        if node_to_get_neighbors.node_pos.x < node_to_get_neighbors.prior_node_pos.x {
            if let Some(west) =
                TilePos::from_i32_pair(origin_tile.x as i32 - 1, origin_tile.y as i32, tilemap_size)
            {
                neighbor_tiles.push(west);
                return neighbor_tiles;
            }
        }

        if node_to_get_neighbors.node_pos.x > node_to_get_neighbors.prior_node_pos.x {
            if let Some(east) =
                TilePos::from_i32_pair(origin_tile.x as i32 + 1, origin_tile.y as i32, tilemap_size)
            {
                neighbor_tiles.push(east);
                return neighbor_tiles;
            }
        }

        if node_to_get_neighbors.node_pos.y < node_to_get_neighbors.prior_node_pos.y {
            if let Some(south) =
                TilePos::from_i32_pair(origin_tile.x as i32, origin_tile.y as i32 - 1, tilemap_size)
            {
                neighbor_tiles.push(south);
                return neighbor_tiles;
            }
        }

        if node_to_get_neighbors.node_pos.y > node_to_get_neighbors.prior_node_pos.y {
            if let Some(north) =
                TilePos::from_i32_pair(origin_tile.x as i32, origin_tile.y as i32 + 1, tilemap_size)
            {
                neighbor_tiles.push(north);
                return neighbor_tiles;
            }
        }
        neighbor_tiles
    }

    fn get_node_mut(&mut self, node_pos: TilePos) -> Option<&mut Node> {
        self.map.get_mut(&node_pos)
    }

    fn new_node(&mut self, new_node_pos: TilePos, prior_node: Node) {
        if !self.map.contains_key(&new_node_pos) {
            let node = Node {
                node_pos: new_node_pos,
                prior_node_pos: prior_node.node_pos,
                move_cost: 0,
                valid_move: false,
                calculated: false,
            };
            self.map.insert(new_node_pos, node);
        }
    }

    fn set_valid_node(&mut self, node_pos: TilePos) -> Result<(), String> {
        return if let Some(node) = self.get_node_mut(node_pos) {
            node.valid_move = true;
            Ok(())
        } else {
            Err(String::from("Error getting node"))
        };
    }

    fn set_calculated_node(&mut self, node_pos: TilePos) -> Result<(), String> {
        return if let Some(node) = self.get_node_mut(node_pos) {
            node.calculated = true;
            Ok(())
        } else {
            Err(String::from("Error getting node"))
        };
    }

    fn get_output(&mut self) -> () {}

    fn get_node(&self, node_pos: TilePos) -> Option<&Node> {
        self.map.get(&node_pos)
    }
}
