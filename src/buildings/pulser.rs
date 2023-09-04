use crate::buildings::{Activate, Building};
use crate::color_system::ColorConflictCallback;
use crate::pathfinding::{check_if_tile_is_colorable, IsColorableNodeCheck, NodeIsPlayersCheck};
use bevy::ecs::system::SystemState;
use bevy::log::info_span;
use bevy::prelude::{Component, Entity, FromReflect, Mut, Query, Reflect, Resource, unwrap, With, Without, World};
use bevy::utils::hashbrown::HashMap;

use crate::mapping::map::MapTileStorage;
use bevy_ecs_tilemap::prelude::{TilePos, TilemapSize};
use bevy_ggf::mapping::MapId;
use bevy_ggf::movement::{TileMoveCheckMeta, TileMoveChecks};
use bevy_ggf::object::ObjectGridPosition;
use bevy_ggf::pathfinding::dijkstra::Node;
use bevy_ggf::pathfinding::{PathfindAlgorithm, PathfindCallback, PathfindMap};
use bevy_ggf::player::PlayerMarker;

#[derive(Resource)]
pub struct PulserQueryState {
    pub query:
        SystemState<Query<'static, 'static, (&'static ObjectGridPosition, &'static PlayerMarker)>>,
}

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct Pulser {
    pub strength: u32,
    pub max_pulse_tiles: u32,
}
// two parts - we pulse outwards, checking the outside neighbors of each tile. If the outside neighbors
// are not the same player then we damage their color by one. Otherwise at that point we stop.
pub fn simulate_pulsers(mut world: &mut World) {
    let mut system_state: SystemState<
        Query<Entity, (Without<MapId>, With<Activate>, With<Building<Pulser>>)>,
    > = SystemState::new(&mut world);
    let pulsers = system_state.get_mut(&mut world);

    let mut pathfind = PulserPathfind { diagonals: false };

    let mut pathfind_map = PulserPathfindMap {
        map: Default::default(),
        diagonals: false,
    };

    let mut tile_move_checks = TileMoveChecks {
        tile_move_checks: vec![
            TileMoveCheckMeta {
                check: Box::new(IsColorableNodeCheck),
            },
            TileMoveCheckMeta {
                check: Box::new(NodeIsPlayersCheck),
            },
        ],
    };

    let pulsers: Vec<Entity> = pulsers.into_iter().collect();

    for entity in pulsers.iter() {
        pathfind.pathfind(
            MapId { id: 1 },
            *entity,
            &mut world,
            &mut tile_move_checks,
            &mut Some(ColorConflictCallback),
            &mut pathfind_map,
        );
        world.entity_mut(*entity).remove::<Activate>();
    }

    system_state.apply(&mut world);
}

pub struct PulserPathfind {
    pub diagonals: bool,
}

impl PathfindAlgorithm<TilePos, Node, Building<Pulser>> for PulserPathfind {
    type PathfindOutput = ();

    fn pathfind<
        CB: PathfindCallback<TilePos>,
        PM: PathfindMap<TilePos, Node, (), Building<Pulser>>,
    >(
        &mut self,
        _: MapId,
        pathfind_entity: Entity,
        world: &mut World,
        node_validity_checks: &mut TileMoveChecks,
        pathfind_callback: &mut Option<CB>,
        pathfind_map: &mut PM,
    ) -> Self::PathfindOutput {
        let my_span = info_span!("Pulser Pathfinding", name = "pulser_pathfinding").entered();
        world.resource_scope(|mut world, maptile_storage: Mut<MapTileStorage>|{
            let mut pulser_query_state = match world.remove_resource::<PulserQueryState>() {
                None => {                    
                    let system_state: SystemState<Query<(&ObjectGridPosition, &PlayerMarker)>> = SystemState::new(world);
                    PulserQueryState{
                    query: system_state,
                }}
                Some(res) => {res}
            };
            let object_query = pulser_query_state.query.get_mut(world);

            let Ok((object_grid_position, player_marker)) = object_query.get(pathfind_entity) else{
                world.insert_resource(pulser_query_state);
                return ();
            };
            let object_grid_position = object_grid_position.clone();
            let player_pathing_id = player_marker.id();

            pathfind_map.new_pathfind_map(object_grid_position.tile_position);

            if let Some(callback) = pathfind_callback {
                let Some(tile_entity) = maptile_storage.tile_storage.get(&object_grid_position.tile_position) else {
                    world.insert_resource(pulser_query_state);
                    return ;
                };
                callback.foreach_tile(
                    pathfind_entity,
                    tile_entity,
                    object_grid_position.tile_position,
                    &mut world,
                );
            }

            let mut available_moves: Vec<TilePos> = vec![];
            let mut tiles_changed: u32 = 0;

            // unvisited nodes
            let mut unvisited_nodes: Vec<Node> = vec![Node {
                node_pos: object_grid_position.tile_position,
                prior_node_pos: object_grid_position.tile_position,
                move_cost: 0,
                valid_move: false,
                calculated: false,
            }];
            let mut visited_nodes: Vec<TilePos> = vec![];
            let pp_loop = info_span!("Pulser Pathfinding", name = "pp_loop").entered();

            // TODO: Create a resource or something that we can use to store all the game stats so that the
            // strength isnt hardcoded anymore
            while !unvisited_nodes.is_empty() && tiles_changed < 2 {
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

                let pp_neighbors = info_span!("Pulser Pathfinding", name = "pp_neighbors").entered();
                'neighbors: for neighbor in neighbors.iter() {
                    if visited_nodes.contains(&neighbor.0) {
                        continue;
                    }

                    pathfind_map.new_node(neighbor.0, current_node);
                    
                    let pp_node_cost_calculation = info_span!("Pulser Pathfinding", name = "pp_node_cost_calculation").entered();
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
                    pp_node_cost_calculation.exit();

                    let pp_node_check_tile_move_checks = info_span!("Pulser Pathfinding", name = "pp_node_check_tile_move_checks").entered();
                    let valid_node = node_validity_checks.check_tile_move_checks(
                        pathfind_entity,
                        neighbor.1,
                        &neighbor.0,
                        &current_node.node_pos,
                        world,
                    );
                    pp_node_check_tile_move_checks.exit();

                    let _ = pathfind_map.set_calculated_node(neighbor.0);
                    if valid_node {
                        let _ = pathfind_map.set_valid_node(neighbor.0);
                        // if none of them return false and cancel the loop then we can infer that we are able to move into that neighbor
                        // we add the neighbor to the list of unvisited nodes and then push the neighbor to the available moves list
                        unvisited_nodes.push(pathfind_map.get_node_mut(neighbor.0).expect(
                            "Is safe because we know we add the node in at the beginning of this loop",
                        ).clone()); //
                        available_moves.push(neighbor.0);
                    }
                    let pp_check_tile_move_checks = info_span!("Pulser Pathfinding", name = "pp_check_if_tile_is_colorable").entered();
                    if check_if_tile_is_colorable(&mut world, neighbor.1, player_pathing_id) {
                        tiles_changed += 1;
                    }
                    pp_check_tile_move_checks.exit();
                    let pp_callback = info_span!("Pulser Pathfinding", name = "pp_callback").entered();
                    if let Some(callback) = pathfind_callback {
                        callback.foreach_tile(pathfind_entity, neighbor.1, neighbor.0, &mut world);
                    }
                    pp_callback.exit();
                }
                pp_neighbors.exit();

                unvisited_nodes.remove(0);
                visited_nodes.push(current_node.node_pos);
            }
            pp_loop.exit();
            world.insert_resource(pulser_query_state);
        });
        ()
    }
}

pub struct PulserPathfindMap {
    pub map: HashMap<TilePos, Node>,
    pub diagonals: bool,
}

impl PathfindMap<TilePos, Node, (), Building<Pulser>> for PulserPathfindMap {
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
        tile_entity: Entity,
        tile_pos: TilePos,
        move_from_tile_pos: TilePos,
        world: &World,
    ) -> bool {
        let Some(object_movement) = world.get::<Building<Pulser>>(entity_moving) else {
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
            <= object_movement.building_type.strength as u32
        {
            tile_node.move_cost = (move_from_tile_node.move_cost + 1);
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

        if self.diagonals {
            if let Some(northwest) = TilePos::from_i32_pair(
                origin_tile.x as i32 - 1,
                origin_tile.y as i32 + 1,
                tilemap_size,
            ) {
                neighbor_tiles.push(northwest);
            }
            if let Some(northeast) = TilePos::from_i32_pair(
                origin_tile.x as i32 + 1,
                origin_tile.y as i32 + 1,
                tilemap_size,
            ) {
                neighbor_tiles.push(northeast);
            }
            if let Some(southeast) = TilePos::from_i32_pair(
                origin_tile.x as i32 + 1,
                origin_tile.y as i32 - 1,
                tilemap_size,
            ) {
                neighbor_tiles.push(southeast);
            }
            if let Some(southwest) = TilePos::from_i32_pair(
                origin_tile.x as i32 - 1,
                origin_tile.y as i32 - 1,
                tilemap_size,
            ) {
                neighbor_tiles.push(southwest);
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
}
