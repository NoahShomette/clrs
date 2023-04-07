use crate::actions::Actions;
use crate::buildings::line::Line;
use crate::buildings::pulser::Pulser;
use crate::buildings::scatter::Scatters;
use crate::buildings::{Building, BuildingCooldown, BuildingMarker, BuildingTypes};
use crate::game::{GameData, BORDER_PADDING_TOTAL};
use crate::player::PlayerPoints;
use bevy::ecs::system::SystemState;
use bevy::prelude::{Color, Entity, Query, Res, ResMut, Timer, TimerMode, UVec2};
use bevy_ascii_terminal::{Terminal, TileFormatter, ToWorld};
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage};
use bevy_ggf::game_core::command::GameCommands;
use bevy_ggf::game_core::state::Changed;
use bevy_ggf::game_core::Game;
use bevy_ggf::mapping::tiles::{ObjectStackingClass, Tile};
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::{Object, ObjectGridPosition, ObjectInfo};
use bevy_ggf::player::{Player, PlayerMarker};
use ns_defaults::camera::CursorWorldPos;

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
                if let Some(world_pos) = to_world.screen_to_world(cursor_world_pos.cursor_world_pos)
                {
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
                                .saturating_sub((BORDER_PADDING_TOTAL / 2) as i32)
                                as u32,
                            y: terminal_pos
                                .y
                                .saturating_sub((BORDER_PADDING_TOTAL / 2) as i32)
                                as u32,
                        };

                        target_tile_pos = TilePos {
                            x: tile_pos.x,
                            y: tile_pos.y,
                        };
                    }
                }
            } else if actions.tile_pos.is_some() {
                target_tile_pos = actions.tile_pos.unwrap();
            } else {
                actions.placed_building = false;
                continue;
            }

            let Some((player_marker, _, _)) = tiles
                        .iter()
                        .find(|(_, id, _)| id == &&target_tile_pos)else{
                        actions.placed_building = false;
                        continue;
                    };

            if player_marker.id() != player_id {
                actions.placed_building = false;
                continue;
            }

            let mut system_state: SystemState<Query<(Entity, &Player, &mut PlayerPoints)>> =
                SystemState::new(&mut game.game_world);
            let mut players = system_state.get_mut(&mut game.game_world);

            let Some((entity, _, mut player_points)) = players
                        .iter_mut()
                        .find(|(_, id, _)| id.id() == player_id)else{
                        actions.placed_building = false;
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
                                        max_pulse_tiles: 3,
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
                                        scatter_amount: 30,
                                    },
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
                                    building_type: Line { strength: 10 },
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
            }
            actions.placed_building = false;
            actions.target_world_pos = false;
            actions.tile_pos = None;
        }
    }
}
