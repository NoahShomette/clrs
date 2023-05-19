use crate::abilities::expand::Expand;
use crate::abilities::fortify::Fortify;
use crate::abilities::nuke::Nuke;
use crate::abilities::{Abilities, Ability, AbilityCooldown, AbilityMarker, SpawnAbilityExt};
use crate::actions::Actions;
use crate::buildings::line::Line;
use crate::buildings::pulser::Pulser;
use crate::buildings::scatter::Scatters;
use crate::buildings::{
    Building, BuildingCooldown, BuildingMarker, BuildingTypes, SpawnBuildingExt,
};
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
use bevy_ggf::mapping::tiles::{ObjectStackingClass, Tile};
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::{Object, ObjectGridPosition, ObjectInfo};
use bevy_ggf::player::{Player, PlayerMarker};
use ns_defaults::camera::CursorWorldPos;

pub fn place_building(
    cursor_world_pos: Res<CursorWorldPos>,
    mut actions: Query<(Option<&PlayerMarker>, Option<&Player>, &mut Actions)>,
    mut term_query: Query<(&mut Terminal, &ToWorld)>,
    mut game_commands: ResMut<GameCommands>,
    game_data: Res<GameData>,
) {
    for (player_marker, player, mut actions) in actions.iter_mut() {
        let player_id;
        if player_marker.is_some() {
            player_id = player_marker.unwrap().id();
        } else {
            player_id = player.unwrap().id();
        }
        if actions.try_place_building {
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
            } else if actions.building_tile_pos.is_some() {
                target_tile_pos = actions.building_tile_pos.unwrap();
            } else {
                continue;
            }


            game_commands.spawn_building(actions.selected_building, player_id, target_tile_pos);
        }
    }
}

pub fn place_ability(
    cursor_world_pos: Res<CursorWorldPos>,
    mut actions: Query<(Option<&PlayerMarker>, Option<&Player>, &mut Actions)>,
    mut term_query: Query<(&mut Terminal, &ToWorld)>,
    mut game_commands: ResMut<GameCommands>,
    game_data: Res<GameData>,
) {
    for (player_marker, player, mut actions) in actions.iter_mut() {
        let player_id;
        if player_marker.is_some() {
            player_id = player_marker.unwrap().id();
        } else {
            player_id = player.unwrap().id();
        }
        if actions.try_place_ability {
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
            } else if actions.ability_tile_pos.is_some() {
                target_tile_pos = actions.ability_tile_pos.unwrap();
            } else {
                continue;
            }

            game_commands.spawn_ability(actions.selected_ability, player_id, target_tile_pos);
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
