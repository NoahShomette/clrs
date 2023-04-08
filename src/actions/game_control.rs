use crate::abilities::expand::Expand;
use crate::abilities::nuke::Nuke;
use crate::abilities::{Abilities, Ability, AbilityCooldown, AbilityMarker};
use crate::actions::Actions;
use crate::buildings::line::Line;
use crate::buildings::pulser::Pulser;
use crate::buildings::scatter::Scatters;
use crate::buildings::{Building, BuildingCooldown, BuildingMarker, BuildingTypes};
use crate::game::{GameData, BORDER_PADDING_TOTAL};
use crate::player::PlayerPoints;
use bevy::ecs::system::SystemState;
use bevy::math::Vec2;
use bevy::prelude::{Color, Entity, Query, Res, ResMut, Timer, TimerMode, UVec2};
use bevy_ascii_terminal::{Terminal, TileFormatter, ToWorld};
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage};
use bevy_ggf::game_core::command::GameCommands;
use bevy_ggf::game_core::state::Changed;
use bevy_ggf::game_core::Game;
use bevy_ggf::mapping::tiles::{ObjectStackingClass, Tile, TileObjectStacks};
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::{Object, ObjectGridPosition, ObjectInfo};
use bevy_ggf::player::{Player, PlayerMarker};
use ns_defaults::camera::CursorWorldPos;
use crate::abilities::fortify::Fortify;

pub fn place_building(
    cursor_world_pos: Res<CursorWorldPos>,
    mut actions: Query<(Option<&PlayerMarker>, Option<&Player>, &mut Actions)>,
    mut term_query: Query<(&mut Terminal, &ToWorld)>,
    tiles: Query<(&PlayerMarker, &TilePos, &Tile)>,
    mut game_commands: ResMut<GameCommands>,
    game_data: Res<GameData>,
    mut game: ResMut<Game>,
) {
    for (player_marker, player, mut actions) in actions.iter_mut() {
        let player_id;
        if player_marker.is_some() {
            player_id = player_marker.unwrap().id();
        } else {
            player_id = player.unwrap().id();
        }
        if actions.placed_building {
            let (mut term, to_world) = term_query.single_mut();
            let mut target_tile_pos = TilePos::default();
            if actions.target_world_pos {
                if let Some(tile_pos) = convert_world_to_game_tile_pos(
                    cursor_world_pos.cursor_world_pos,
                    &game_data,
                    &to_world,
                    &mut term,
                ) {
                    target_tile_pos = tile_pos;
                } else {
                    continue;
                }
            } else if actions.tile_pos.is_some() {
                target_tile_pos = actions.tile_pos.unwrap();
            } else {
                continue;
            }

            let Some((player_marker, _, _)) = tiles
                        .iter()
                        .find(|(_, id, _)| id == &&target_tile_pos)else{
                        continue;
                    };

            if player_marker.id() != player_id {
                continue;
            }

            let mut system_state: SystemState<Query<(Entity, &Player, &mut PlayerPoints)>> =
                SystemState::new(&mut game.game_world);
            let mut players = system_state.get_mut(&mut game.game_world);

            let Some((entity, _, mut player_points)) = players
                        .iter_mut()
                        .find(|(_, id, _)| id.id() == player_id)else{
                        continue;
                    };

            match actions.selected_building {
                BuildingTypes::Pulser => {
                    if player_points.building_points >= 50 {
                        let _ = game_commands.spawn_object(
                            (
                                ObjectGridPosition {
                                    tile_position: target_tile_pos,
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
                                    object_type: game_data
                                        .object_types
                                        .get("Pulser")
                                        .unwrap()
                                        .clone(),
                                },
                                Building {
                                    building_type: Pulser {
                                        strength: 7,
                                        max_pulse_tiles: 2,
                                    },
                                },
                                BuildingCooldown {
                                    timer: Timer::from_seconds(0.1, TimerMode::Once),
                                    timer_reset: 0.1,
                                },
                                BuildingMarker::default(),
                            ),
                            target_tile_pos,
                            MapId { id: 1 },
                            player_id,
                        );
                        player_points.building_points =
                            player_points.building_points.saturating_sub(50);
                        game.game_world
                            .entity_mut(entity)
                            .insert(Changed::default());
                    }
                }
                BuildingTypes::Scatter => {
                    if player_points.building_points >= 50 {
                        let _ = game_commands.spawn_object(
                            (
                                ObjectGridPosition {
                                    tile_position: target_tile_pos,
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
                                    object_type: game_data
                                        .object_types
                                        .get("Scatter")
                                        .unwrap()
                                        .clone(),
                                },
                                Building {
                                    building_type: Scatters {
                                        scatter_range: 3,
                                        scatter_amount: 20,
                                    },
                                },
                                BuildingCooldown {
                                    timer: Timer::from_seconds(0.15, TimerMode::Once),
                                    timer_reset: 0.15,
                                },
                                BuildingMarker::default(),
                            ),
                            target_tile_pos,
                            MapId { id: 1 },
                            player_id,
                        );
                        player_points.building_points =
                            player_points.building_points.saturating_sub(50);
                        game.game_world
                            .entity_mut(entity)
                            .insert(Changed::default());
                    }
                }
                BuildingTypes::Line => {
                    if player_points.building_points >= 50 {
                        let _ = game_commands.spawn_object(
                            (
                                ObjectGridPosition {
                                    tile_position: target_tile_pos,
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
                                    object_type: game_data
                                        .object_types
                                        .get("Line")
                                        .unwrap()
                                        .clone(),
                                },
                                Building {
                                    building_type: Line { strength: 8 },
                                },
                                BuildingCooldown {
                                    timer: Timer::from_seconds(0.2, TimerMode::Once),
                                    timer_reset: 0.2,
                                },
                                BuildingMarker::default(),
                            ),
                            target_tile_pos,
                            MapId { id: 1 },
                            player_id,
                        );
                        player_points.building_points =
                            player_points.building_points.saturating_sub(50);
                        game.game_world
                            .entity_mut(entity)
                            .insert(Changed::default());
                    }
                }
            }
        }
    }
}

pub fn place_ability(
    cursor_world_pos: Res<CursorWorldPos>,
    mut actions: Query<(Option<&PlayerMarker>, Option<&Player>, &mut Actions)>,
    mut term_query: Query<(&mut Terminal, &ToWorld)>,
    tiles: Query<(&PlayerMarker, &TilePos, &Tile)>,
    mut game_commands: ResMut<GameCommands>,
    game_data: Res<GameData>,
    mut game: ResMut<Game>,
) {
    for (player_marker, player, mut actions) in actions.iter_mut() {
        let player_id;
        if player_marker.is_some() {
            player_id = player_marker.unwrap().id();
        } else {
            player_id = player.unwrap().id();
        }
        if actions.placed_ability {
            let (mut term, to_world) = term_query.single_mut();
            let mut target_tile_pos = TilePos::default();
            if actions.target_world_pos {
                if let Some(tile_pos) = convert_world_to_game_tile_pos(
                    cursor_world_pos.cursor_world_pos,
                    &game_data,
                    &to_world,
                    &mut term,
                ) {
                    target_tile_pos = tile_pos;
                } else {
                    continue;
                }
            } else if actions.tile_pos.is_some() {
                target_tile_pos = actions.tile_pos.unwrap();
            } else {
                continue;
            }

            let mut system_state: SystemState<Query<(Entity, &Player, &mut PlayerPoints)>> =
                SystemState::new(&mut game.game_world);
            let mut players = system_state.get_mut(&mut game.game_world);

            let Some((entity, _, mut player_points)) = players
                .iter_mut()
                .find(|(_, id, _)| id.id() == player_id)else{
                continue;
            };

            match actions.selected_ability {
                Abilities::Nuke => {
                    if player_points.ability_points >= 50 {
                        let _ = game_commands.spawn_object(
                            (
                                ObjectGridPosition {
                                    tile_position: target_tile_pos,
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
                                    object_type: game_data
                                        .object_types
                                        .get("Nuke")
                                        .unwrap()
                                        .clone(),
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
                            target_tile_pos,
                            MapId { id: 1 },
                            player_id,
                        );
                        player_points.ability_points =
                            player_points.ability_points.saturating_sub(50);
                        game.game_world
                            .entity_mut(entity)
                            .insert(Changed::default());
                    }
                }
                Abilities::Fortify => {
                    let Some((player_marker, _, _)) = tiles
                        .iter()
                        .find(|(_, id, _)| id == &&target_tile_pos)else{
                        continue;
                    };

                    if player_points.ability_points >= 50 && player_marker.id() == player_id {
                        let _ = game_commands.spawn_object(
                            (
                                ObjectGridPosition {
                                    tile_position: target_tile_pos,
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
                                    object_type: game_data
                                        .object_types
                                        .get("Fortify")
                                        .unwrap()
                                        .clone(),
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
                            target_tile_pos,
                            MapId { id: 1 },
                            player_id,
                        );
                        player_points.ability_points =
                            player_points.ability_points.saturating_sub(50);
                        game.game_world
                            .entity_mut(entity)
                            .insert(Changed::default());
                    }
                }
                Abilities::Expand => {
                    if player_points.ability_points >= 50 {
                        let _ = game_commands.spawn_object(
                            (
                                ObjectGridPosition {
                                    tile_position: target_tile_pos,
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
                                    object_type: game_data
                                        .object_types
                                        .get("Expand")
                                        .unwrap()
                                        .clone(),
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
                            target_tile_pos,
                            MapId { id: 1 },
                            player_id,
                        );
                        player_points.ability_points =
                            player_points.ability_points.saturating_sub(50);
                        game.game_world
                            .entity_mut(entity)
                            .insert(Changed::default());
                    }
                }
            }
        }
    }
}

pub fn convert_world_to_game_tile_pos(
    world_pos: Vec2,
    game_data: &Res<GameData>,
    to_world: &ToWorld,
    term: &mut Terminal,
) -> Option<TilePos> {
    if let Some(world_pos) = to_world.screen_to_world(world_pos) {
        let terminal_pos = to_world.world_to_tile(world_pos);
        if terminal_pos.x >= (BORDER_PADDING_TOTAL / 2) as i32
            && terminal_pos
                .x
                .saturating_sub((BORDER_PADDING_TOTAL / 2) as i32)
                < (game_data.map_size_x) as i32
            && terminal_pos.y >= (BORDER_PADDING_TOTAL / 2) as i32
            && terminal_pos
                .y
                .saturating_sub((BORDER_PADDING_TOTAL / 2) as i32)
                < (game_data.map_size_y) as i32
        {
            term.put_char(terminal_pos, 'X'.fg(Color::GREEN));

            let tile_pos: UVec2 = UVec2 {
                x: terminal_pos
                    .x
                    .saturating_sub((BORDER_PADDING_TOTAL / 2) as i32) as u32,
                y: terminal_pos
                    .y
                    .saturating_sub((BORDER_PADDING_TOTAL / 2) as i32) as u32,
            };
            return Some(TilePos {
                x: tile_pos.x,
                y: tile_pos.y,
            });
        } else {
            return None;
        }
    }

    return None;
}
