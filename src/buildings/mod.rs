pub mod line;
pub mod pulser;
pub mod scatter;

use crate::buildings::line::Line;
use crate::buildings::pulser::Pulser;
use crate::buildings::scatter::Scatters;
use crate::game::GameData;
use crate::player::PlayerPoints;
use bevy::ecs::system::SystemState;
use bevy::prelude::{
    Bundle, Commands, Component, Entity, FromReflect, Query, Reflect, Res, ResMut, Timer, With,
    Without, World,
};
use bevy::time::{Time, TimerMode};
use bevy::utils::hashbrown::HashMap;
use bevy_ecs_tilemap::prelude::{TileStorage, TilemapSize};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ggf::game_core::command::{GameCommand, GameCommands};
use bevy_ggf::game_core::state::{Changed, DespawnedObjects};
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::{ObjectStackingClass, Tile, TileObjectStacks};
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::{Object, ObjectGridPosition, ObjectId, ObjectInfo};
use bevy_ggf::player::{Player, PlayerMarker};

pub trait SpawnBuildingExt {
    fn spawn_building(
        &mut self,
        ability: BuildingTypes,
        player_id: usize,
        target_tile: TilePos,
    ) -> SpawnBuilding;
}

impl SpawnBuildingExt for GameCommands {
    fn spawn_building(
        &mut self,
        ability: BuildingTypes,
        player_id: usize,
        target_tile: TilePos,
    ) -> SpawnBuilding {
        self.queue.push(SpawnBuilding {
            building_type: ability.clone(),
            player_id: player_id.clone(),
            target_tile_pos: target_tile.clone(),
        });
        SpawnBuilding {
            building_type: ability.clone(),
            player_id: player_id.clone(),
            target_tile_pos: target_tile.clone(),
        }
    }
}

#[derive(Reflect, FromReflect, Clone)]
pub struct SpawnBuilding {
    pub building_type: BuildingTypes,
    pub player_id: usize,
    pub target_tile_pos: TilePos,
}

impl GameCommand for SpawnBuilding {
    fn execute(&mut self, world: &mut World) -> Result<(), String> {
        let game_data = world.remove_resource::<GameData>().unwrap();

        let mut system_state: SystemState<(
            Query<(Entity, &Player, &mut PlayerPoints)>,
            Query<(&PlayerMarker, &TilePos, &Tile, &TileObjectStacks)>,
        )> = SystemState::new(world);
        let (mut players, tiles) = system_state.get_mut(world);

        let Some((entity, _, mut player_points)) = players
            .iter_mut()
            .find(|(_, id, _)| id.id() == self.player_id)else {
            world.insert_resource(game_data);
            return Err("Failed to Find Player ID".parse().unwrap());
        };

        let Some((player_marker, _, _, tile_object_stacks)) = tiles
            .iter()
            .find(|(_, id, _, _)| id == &&self.target_tile_pos)else {
            world.insert_resource(game_data);
            return Err("Failed to Find target tile pos".parse().unwrap());
        };

        if player_marker.id() != self.player_id {
            world.insert_resource(game_data);
            return Err("Tile not owned by placing player".parse().unwrap());
        }

        if !tile_object_stacks.has_space(&ObjectStackingClass {
            stack_class: game_data.stacking_classes.get("Building").unwrap().clone(),
        }) {
            world.insert_resource(game_data);
            return Err("Tile already occupied".parse().unwrap());
        }

        let mut game_commands = GameCommands::new();

        let result = match self.building_type {
            BuildingTypes::Pulser => {
                if player_points.building_points >= 50 {
                    //actions.placed_building = true;

                    player_points.building_points =
                        player_points.building_points.saturating_sub(50);
                    world.entity_mut(entity).insert(Changed::default());

                    let mut spawn = game_commands.spawn_object(
                        (
                            ObjectGridPosition {
                                tile_position: self.target_tile_pos,
                            },
                            ObjectStackingClass {
                                stack_class: game_data
                                    .stacking_classes
                                    .get("Building")
                                    .unwrap()
                                    .clone(),
                            },
                            Object,
                            ObjectInfo {
                                object_type: game_data.object_types.get("Pulser").unwrap().clone(),
                            },
                            Building {
                                building_type: Pulser {
                                    strength: 7,
                                    max_pulse_tiles: 2,
                                },
                            },
                            BuildingCooldown {
                                timer: Timer::from_seconds(0.15, TimerMode::Once),
                                timer_reset: 0.15,
                            },
                            BuildingMarker::default(),
                        ),
                        self.target_tile_pos,
                        MapId { id: 1 },
                        self.player_id,
                    );

                    spawn.execute(world)
                } else {
                    Err(String::from("Not enough points to place"))
                }
            }
            BuildingTypes::Scatter => {
                if player_points.building_points >= 50 {
                    //actions.placed_building = true;

                    player_points.building_points =
                        player_points.building_points.saturating_sub(50);
                    world.entity_mut(entity).insert(Changed::default());

                    let mut spawn = game_commands.spawn_object(
                        (
                            ObjectGridPosition {
                                tile_position: self.target_tile_pos,
                            },
                            ObjectStackingClass {
                                stack_class: game_data
                                    .stacking_classes
                                    .get("Building")
                                    .unwrap()
                                    .clone(),
                            },
                            Object,
                            ObjectInfo {
                                object_type: game_data.object_types.get("Scatter").unwrap().clone(),
                            },
                            Building {
                                building_type: Scatters {
                                    scatter_range: 3,
                                    scatter_amount: 20,
                                },
                            },
                            BuildingCooldown {
                                timer: Timer::from_seconds(0.13, TimerMode::Once),
                                timer_reset: 0.13,
                            },
                            BuildingMarker::default(),
                        ),
                        self.target_tile_pos,
                        MapId { id: 1 },
                        self.player_id,
                    );
                    spawn.execute(world)
                } else {
                    Err(String::from("Not enough points to place"))
                }
            }
            BuildingTypes::Line => {
                if player_points.building_points >= 50 {
                    //actions.placed_building = true;

                    player_points.building_points =
                        player_points.building_points.saturating_sub(50);
                    world.entity_mut(entity).insert(Changed::default());

                    let mut spawn = game_commands.spawn_object(
                        (
                            ObjectGridPosition {
                                tile_position: self.target_tile_pos,
                            },
                            ObjectStackingClass {
                                stack_class: game_data
                                    .stacking_classes
                                    .get("Building")
                                    .unwrap()
                                    .clone(),
                            },
                            Object,
                            ObjectInfo {
                                object_type: game_data.object_types.get("Line").unwrap().clone(),
                            },
                            Building {
                                building_type: Line { strength: 8 },
                            },
                            BuildingCooldown {
                                timer: Timer::from_seconds(0.15, TimerMode::Once),
                                timer_reset: 0.15,
                            },
                            BuildingMarker::default(),
                        ),
                        self.target_tile_pos,
                        MapId { id: 1 },
                        self.player_id,
                    );

                    spawn.execute(world)
                } else {
                    Err(String::from("Not enough points to place"))
                }
            }
        };

        world.insert_resource(game_data);

        return result;
    }
}

pub fn destroy_buildings(
    buildings: Query<
        (
            Entity,
            &PlayerMarker,
            &ObjectId,
            &ObjectGridPosition,
            &BuildingMarker,
        ),
        With<Object>,
    >,
    mut tiles: Query<(Entity, &TileTerrainInfo, &PlayerMarker), (Without<Object>, With<Tile>)>,
    mut tile_storage_query: Query<(&MapId, &TileStorage)>,
    mut commands: Commands,
    mut despawn_objects: ResMut<DespawnedObjects>,
) {
    for (building_entity, player_marker, object_id, object_grid_pos, building) in buildings.iter() {
        let Some((_, tile_storage)) = tile_storage_query
            .iter_mut()
            .find(|(id, _)| id == &&MapId { id: 1 })else {
            continue;
        };

        let tile_entity = tile_storage.get(&object_grid_pos.tile_position).unwrap();

        let Ok((entity, tile_terrain_info, tile_marker)) = tiles.get_mut(tile_entity) else {
            continue;
        };

        if player_marker != tile_marker
            || tile_terrain_info.terrain_type.terrain_class.name.as_str() == "NonColorable"
        {
            println!("killing buildings");
            despawn_objects
                .despawned_objects
                .insert(*object_id, Changed::default());
            commands.entity(building_entity).despawn();
        }
    }
}

pub fn update_building_timers(
    mut timers: Query<(Entity, &mut BuildingCooldown), Without<Activate>>,
    mut commands: Commands,
    mut time: Res<Time>,
) {
    for (entity, mut timer) in timers.iter_mut() {
        timer.timer.tick(time.delta());
        if timer.timer.finished() {
            commands.entity(entity).insert(Activate);
            timer.timer = Timer::from_seconds(timer.timer_reset, TimerMode::Once);
        }
    }
}

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct BuildingMarker;

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct Building<T> {
    pub building_type: T,
}

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct Activate;

#[derive(Default, Clone, Debug, Component, Reflect, FromReflect)]
pub struct BuildingCooldown {
    pub timer: Timer,
    pub timer_reset: f32,
}

#[derive(
    Default, Clone, Copy, Eq, Hash, Debug, PartialEq, serde::Deserialize, Reflect, FromReflect,
)]
pub enum BuildingTypes {
    #[default]
    Pulser,
    Scatter,
    Line,
}

#[derive(Default, Clone, Copy, Eq, Hash, Debug, PartialEq)]
pub struct TileNode {
    pub(crate) cost: Option<u32>,
    pub(crate) prior_node: TilePos,
    pub(crate) tile_pos: TilePos,
}

pub fn check_is_colorable(tile_terrain_info: &TileTerrainInfo) -> bool {
    tile_terrain_info.terrain_type.name == String::from("BasicColorable")
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

pub fn tile_cost_check(
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
pub fn get_two_node_mut(
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
