pub mod expand;
pub mod fortify;
pub mod nuke;

use crate::abilities::expand::Expand;
use crate::abilities::fortify::Fortify;
use crate::abilities::nuke::Nuke;
use crate::buildings::Activate;
use crate::game::GameData;
use crate::player::PlayerPoints;
use bevy::ecs::system::SystemState;
use bevy::prelude::{
    Commands, Component, Entity, FromReflect, Query, Reflect, Res, ResMut, Time, Timer, TimerMode,
    With, Without, World,
};
use bevy_ecs_tilemap::prelude::TileStorage;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ggf::game_core::command::{GameCommand, GameCommands};
use bevy_ggf::game_core::state::{Changed, DespawnedObjects};
use bevy_ggf::mapping::terrain::{TerrainClass, TileTerrainInfo};
use bevy_ggf::mapping::tiles::{ObjectStackingClass, Tile, TileObjectStacks};
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::{Object, ObjectGridPosition, ObjectId, ObjectInfo};
use bevy_ggf::player::{Player, PlayerMarker};

pub trait SpawnAbilityExt {
    fn spawn_ability(
        &mut self,
        ability: Abilities,
        player_id: usize,
        target_tile: TilePos,
    ) -> SpawnAbility;
}

impl SpawnAbilityExt for GameCommands {
    fn spawn_ability(
        &mut self,
        ability: Abilities,
        player_id: usize,
        target_tile: TilePos,
    ) -> SpawnAbility {
        self.queue.push(SpawnAbility {
            ability_type: ability.clone(),
            player_id: player_id.clone(),
            target_tile_pos: target_tile.clone(),
        });
        SpawnAbility {
            ability_type: ability.clone(),
            player_id: player_id.clone(),
            target_tile_pos: target_tile.clone(),
        }
    }
}

#[derive(Reflect, FromReflect, Clone)]
pub struct SpawnAbility {
    pub ability_type: Abilities,
    pub player_id: usize,
    pub target_tile_pos: TilePos,
}

impl GameCommand for SpawnAbility {
    fn execute(&mut self, world: &mut World) -> Result<(), String> {
        let game_data = world.remove_resource::<GameData>().unwrap();

        let mut system_state: SystemState<(
            Query<(Entity, &Player, &mut PlayerPoints)>,
            Query<(Option<&PlayerMarker>, &TilePos, &Tile, &TileTerrainInfo)>,
        )> = SystemState::new(world);
        let (mut players, tiles) = system_state.get_mut(world);

        let Some((entity, _, mut player_points)) = players
            .iter_mut()
            .find(|(_, id, _)| id.id() == self.player_id)else{
            world.insert_resource(game_data);
            return Err("Failed to Find Player ID".parse().unwrap())
        };

        let mut game_commands = GameCommands::new();

        let result = match self.ability_type {
            Abilities::Nuke => {
                if player_points.ability_points >= 50 {
                    //actions.placed_ability = true;

                    player_points.ability_points = player_points.ability_points.saturating_sub(50);

                    world.entity_mut(entity).insert(Changed::default());

                    let mut spawn = game_commands.spawn_object(
                        (
                            ObjectGridPosition {
                                tile_position: self.target_tile_pos,
                            },
                            ObjectStackingClass {
                                stack_class: game_data
                                    .stacking_classes
                                    .get("Ability")
                                    .unwrap()
                                    .clone(),
                            },
                            Object,
                            ObjectInfo {
                                object_type: game_data.object_types.get("Nuke").unwrap().clone(),
                            },
                            Ability {
                                ability_type: Nuke {
                                    strength: 5,
                                    min_tile_damage: 2,
                                    max_tile_damage: 4,
                                },
                            },
                            AbilityCooldown {
                                timer: Timer::from_seconds(0.3, TimerMode::Once),
                                timer_reset: 0.3,
                                timer_ticks: 2,
                            },
                            AbilityMarker {
                                requires_player_territory: false,
                            },
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
            Abilities::Fortify => {
                let Some((player_marker, _, _, tile_terrain_info)) = tiles
                    .iter()
                    .find(|(_, id, _, _)| id == &&self.target_tile_pos)else {
                    
                    world.insert_resource(game_data);
                    return Err("Failed to Find Fortify Tile Pos".parse().unwrap())
                };
                
                let Some(player_marker) = player_marker else {
                    world.insert_resource(game_data);
                    return Err("Tile doesnt contain a player marker".parse().unwrap())
                };

                if tile_terrain_info.terrain_type.terrain_class
                    != (TerrainClass {
                    name: "Colorable".to_string(),
                })
                {
                    world.insert_resource(game_data);
                    return Err("Tile is not a Colorable Tile".parse().unwrap());
                }

                if player_points.ability_points >= 50 && player_marker.id() == self.player_id {
                    //actions.placed_ability = true;

                    player_points.ability_points = player_points.ability_points.saturating_sub(50);
                    world.entity_mut(entity).insert(Changed::default());

                    let mut spawn = game_commands.spawn_object(
                        (
                            ObjectGridPosition {
                                tile_position: self.target_tile_pos,
                            },
                            ObjectStackingClass {
                                stack_class: game_data
                                    .stacking_classes
                                    .get("Ability")
                                    .unwrap()
                                    .clone(),
                            },
                            Object,
                            ObjectInfo {
                                object_type: game_data.object_types.get("Fortify").unwrap().clone(),
                            },
                            Ability {
                                ability_type: Fortify {
                                    strength: 5,
                                    min_tile_strengthen: 3,
                                    max_tile_strengthen: 5,
                                },
                            },
                            AbilityCooldown {
                                timer: Timer::from_seconds(0.3, TimerMode::Once),
                                timer_reset: 0.3,
                                timer_ticks: 20,
                            },
                            AbilityMarker {
                                requires_player_territory: false,
                            },
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
            Abilities::Expand => {

                let Some((_, _, _, tile_terrain_info)) = tiles
                    .iter()
                    .find(|(_, id, _, _)| id == &&self.target_tile_pos)else {

                    world.insert_resource(game_data);
                    return Err("Failed to Find Fortify Tile Pos".parse().unwrap())
                };

                if tile_terrain_info.terrain_type.terrain_class
                    != (TerrainClass {
                    name: "Colorable".to_string(),
                })
                {
                    world.insert_resource(game_data);
                    return Err("Tile is not a Colorable Tile".parse().unwrap());
                }
                
                if player_points.ability_points >= 50 {
                    //actions.placed_ability = true;

                    player_points.ability_points = player_points.ability_points.saturating_sub(50);
                    world.entity_mut(entity).insert(Changed::default());

                    let mut spawn = game_commands.spawn_object(
                        (
                            ObjectGridPosition {
                                tile_position: self.target_tile_pos,
                            },
                            ObjectStackingClass {
                                stack_class: game_data
                                    .stacking_classes
                                    .get("Ability")
                                    .unwrap()
                                    .clone(),
                            },
                            Object,
                            ObjectInfo {
                                object_type: game_data.object_types.get("Expand").unwrap().clone(),
                            },
                            Ability {
                                ability_type: Expand {
                                    strength: 2,
                                    min_tile_strengthen: 1,
                                    max_tile_strengthen: 2,
                                },
                            },
                            AbilityCooldown {
                                timer: Timer::from_seconds(0.1, TimerMode::Once),
                                timer_reset: 0.1,
                                timer_ticks: 10,
                            },
                            AbilityMarker {
                                requires_player_territory: false,
                            },
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

pub fn destroy_abilities(
    abilities: Query<
        (
            Entity,
            &ObjectId,
            &AbilityMarker,
            &ObjectGridPosition,
            &ObjectStackingClass,
        ),
        (With<Object>, With<DestroyAbility>),
    >,
    mut tiles: Query<(Entity, &mut TileObjectStacks), (Without<Object>, With<Tile>)>,
    mut tile_storage_query: Query<(&MapId, &TileStorage)>,
    mut commands: Commands,
    mut despawn_objects: ResMut<DespawnedObjects>,
) {
    for (building_entity, object_id, ability, object_grid_pos, object_stacking_class) in
        abilities.iter()
    {
        let Some((_, tile_storage)) = tile_storage_query
            .iter_mut()
            .find(|(id, _)| id == &&MapId { id: 1 })else {
            continue;
        };

        let tile_entity = tile_storage.get(&object_grid_pos.tile_position).unwrap();

        let Ok((entity, mut tile_object_stacks)) = tiles.get_mut(tile_entity) else {
            continue;
        };

        despawn_objects
            .despawned_objects
            .insert(*object_id, Changed::default());
        commands.entity(building_entity).despawn();
        tile_object_stacks.decrement_object_class_count(object_stacking_class);
    }
}

pub fn update_ability_timers(
    mut timers: Query<(Entity, &mut AbilityCooldown), Without<Activate>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut timer) in timers.iter_mut() {
        timer.timer.tick(time.delta());
        if timer.timer.finished() {
            commands.entity(entity).insert(Activate);
            timer.timer = Timer::from_seconds(timer.timer_reset, TimerMode::Once);
            timer.timer_ticks = timer.timer_ticks.saturating_sub(1);
        }
    }
}

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct DestroyAbility;

#[derive(
    Default, Clone, Copy, Eq, Hash, Debug, PartialEq, serde::Deserialize, Reflect, FromReflect,
)]
pub enum Abilities {
    #[default]
    Nuke,
    Fortify,
    Expand,
}

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct AbilityMarker {
    pub requires_player_territory: bool,
}

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct Ability<T> {
    pub ability_type: T,
}

#[derive(Default, Clone, Debug, Component, Reflect, FromReflect)]
pub struct AbilityCooldown {
    pub timer: Timer,
    pub timer_ticks: u32,
    pub timer_reset: f32,
}
