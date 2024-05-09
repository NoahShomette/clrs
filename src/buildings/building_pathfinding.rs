//! Core implementation of [`PathfindAlgorithm`] for use by any buildings

use std::{collections::BTreeMap, marker::PhantomData};

use bevy::{ecs::{component::Component, entity::Entity, system::{Query, Resource, SystemState}, world::{Mut, World}}, utils::HashMap};
use bevy_ecs_tilemap::{map::TilemapSize, tiles::TilePos};
use bevy_ggf::{mapping::MapId, movement::TileMoveChecks, object::{ObjectGridPosition, ObjectId}, pathfinding::{dijkstra::Node, MapNode, PathfindAlgorithm, PathfindCallback, PathfindMap}, player::PlayerMarker};

use crate::{abilities::Ability, mapping::map::MapTileStorage, objects::TileToObjectIndex, pathfinding::{AddObjectToTileToObjectIndex, RemoveObjectFromTileToObjectIndex}};

use super::Building;



#[derive(Resource)]
pub struct BuildingQueryState<BuildingType: Send + Sync + 'static + Component> {
    pub query:
        SystemState<Query<'static, 'static, (&'static ObjectGridPosition, &'static PlayerMarker, &'static BuildingType)>>,
}


#[derive(Default)]
pub struct SimplePathfinder<BuildingType>{
    pd: PhantomData<BuildingType>
}

impl<BuildingType: Send + Sync + 'static + Component> PathfindAlgorithm<TilePos, Node, BuildingType> for SimplePathfinder<BuildingType> {
    type PathfindOutput = ();

    fn pathfind<
        CB: PathfindCallback<TilePos>,
        PM: PathfindMap<TilePos, Node, (), BuildingType>,
    >(
        &mut self,
        _: MapId,
        pathfind_entity: Entity,
        world: &mut World,
        node_validity_checks: &mut TileMoveChecks,
        pathfind_callback: &mut Option<CB>,
        pathfind_map: &mut PM,
    ) -> Self::PathfindOutput {
        world.resource_scope(|mut world, maptile_storage: Mut<MapTileStorage>|{
            let mut pulser_query_state = match world.remove_resource::<BuildingQueryState<BuildingType>>() {
                None => {                    
                    let system_state: SystemState<Query<(&ObjectGridPosition, &PlayerMarker, &BuildingType)>> = SystemState::new(world);
                    BuildingQueryState{
                    query: system_state,
                }}
                Some(res) => {res}
            };
            let object_query = pulser_query_state.query.get_mut(world);

            let Ok((object_grid_position, _, _)) = object_query.get(pathfind_entity) else{
                world.insert_resource(pulser_query_state);
                return ();
            };
            let object_grid_position = object_grid_position.clone();

            pathfind_map.new_pathfind_map(object_grid_position.tile_position.into());

            if let Some(callback) = pathfind_callback {
                let Some(tile_entity) = maptile_storage.tile_storage.get(&object_grid_position.tile_position.into()) else {
                    world.insert_resource(pulser_query_state);
                    return ;
                };
                callback.foreach_tile(
                    pathfind_entity,
                    tile_entity,
                    object_grid_position.tile_position.into(),
                    pathfind_map.get_node(object_grid_position.tile_position.into()).unwrap().cost(),
                    &mut world,
                );
            }

            let mut available_moves: Vec<TilePos> = vec![];

            // unvisited nodes
            let mut unvisited_nodes: Vec<Node> = vec![Node {
                node_pos: object_grid_position.tile_position.into(),
                prior_node_pos: object_grid_position.tile_position.into(),
                move_cost: 0,
                valid_move: false,
                calculated: false,
            }];
            let mut visited_nodes: Vec<TilePos> = vec![];

            // TODO: Create a resource or something that we can use to store all the game stats so that the
            // strength isnt hardcoded anymore
            while !unvisited_nodes.is_empty(){
                unvisited_nodes.sort_by(|x, y| x.move_cost.partial_cmp(&y.move_cost).unwrap());

                let Some(current_node) = unvisited_nodes.get(0) else {
                    continue;
                };

                let neighbor_pos = pathfind_map.get_neighbors(current_node.node_pos, &maptile_storage.tilemap_size.clone());

                let current_node = *current_node;
                let mut neighbors: Vec<(TilePos, Entity)> = vec![];
                for neighbor in neighbor_pos.iter() {
                    let Some(tile_entity) = maptile_storage.tile_storage.get(neighbor) else {
                        continue;
                    };
                    neighbors.push((*neighbor, tile_entity));
                }

                'neighbors: for neighbor in neighbors.iter() {
                    if visited_nodes.contains(&neighbor.0) {
                        continue;
                    }

                    pathfind_map.new_node(neighbor.0, current_node);
                    
                    if !pathfind_map.node_cost_calculation(
                        pathfind_entity,
                        neighbor.1,
                        neighbor.0,
                        current_node.node_pos,
                        world,
                    ) {
                        let _ = pathfind_map.set_calculated_node(neighbor.0);
                        continue 'neighbors;
                    }

                    let valid_node = node_validity_checks.check_tile_move_checks(
                        pathfind_entity,
                        neighbor.1,
                        &neighbor.0,
                        &current_node.node_pos,
                        world,
                    );

                    if valid_node {
                        let _ = pathfind_map.set_valid_node(neighbor.0);
                        // if none of them return false and cancel the loop then we can infer that we are able to move into that neighbor
                        // we add the neighbor to the list of unvisited nodes and then push the neighbor to the available moves list
                        unvisited_nodes.push(pathfind_map.get_node_mut(neighbor.0).expect(
                            "Is safe because we know we add the node in at the beginning of this loop",
                        ).clone()); //
                        available_moves.push(neighbor.0);
                    }

                    if let Some(callback) = pathfind_callback {
                        callback.foreach_tile(pathfind_entity, neighbor.1,  neighbor.0,pathfind_map.get_node(neighbor.0).unwrap().cost(), &mut world);
                    }
                }

                unvisited_nodes.remove(0);
                visited_nodes.push(current_node.node_pos);
            }
            world.insert_resource(pulser_query_state);
        });
        ()
    }
}


#[derive(Default)]
pub struct SimplePathfindMap<BuildingType: PathfindStrengthExt> {
    pub map: HashMap<TilePos, Node>,
    pd: PhantomData<BuildingType>
}

pub trait PathfindStrengthExt{
    fn pathfinding_strength(&self) -> u32;
}

impl<T> PathfindStrengthExt for Building<T> where T: PathfindStrengthExt{
    fn pathfinding_strength(&self) -> u32 {
        self.building_type.pathfinding_strength()
    }
}

impl<T> PathfindStrengthExt for Ability<T> where T: PathfindStrengthExt{
    fn pathfinding_strength(&self) -> u32 {
        self.ability_type.pathfinding_strength()
    }
}

impl<BuildingType: PathfindStrengthExt> RemoveObjectFromTileToObjectIndex for SimplePathfindMap<BuildingType>{
    fn remove_from_index(&mut self, object_id: ObjectId, tile_to_object_index: &mut TileToObjectIndex) {
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

impl<BuildingType: PathfindStrengthExt> AddObjectToTileToObjectIndex for SimplePathfindMap<BuildingType>{
    fn add_to_index(&mut self, object_id: ObjectId, tile_to_object_index: &mut TileToObjectIndex, btree_cache: &mut BTreeMap<u8, Vec<TilePos>>) {
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


impl<BuildingType: Send + Sync + 'static + PathfindStrengthExt + Component> PathfindMap<TilePos, Node, (), BuildingType> for SimplePathfindMap<BuildingType> {
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
        let Some(object_movement) = world.get::<BuildingType>(entity_moving) else {
            return false;
        };

        let Some([tile_node, move_from_tile_node]) =
            self.map.get_many_mut([&tile_pos, &move_from_tile_pos]) else{
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
            <= object_movement.pathfinding_strength()
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
