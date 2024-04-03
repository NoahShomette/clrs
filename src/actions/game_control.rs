use crate::abilities::SpawnAbilityExt;
use crate::actions::Actions;

use crate::buildings::SpawnBuildingExt;
use crate::draw::world_pos_to_tile_pos;
use crate::game::GameData;
use bevy::prelude::{Query, Res, ResMut};
use bevy_ecs_tilemap::prelude::{TilePos, TilemapSize};
use bevy_ggf::game_core::command::GameCommands;
use bevy_ggf::player::{Player, PlayerMarker};
use ns_defaults::camera::CursorWorldPos;

pub fn place_building(
    cursor_world_pos: Res<CursorWorldPos>,
    mut actions: Query<(Option<&PlayerMarker>, Option<&Player>, &mut Actions)>,
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
            let mut target_tile_pos = TilePos::default();
            //println!("{:?}", cursor_world_pos.cursor_world_pos);
            if actions.target_world_pos {
                if let Some(tile_pos) = world_pos_to_tile_pos(
                    &cursor_world_pos.cursor_world_pos,
                    &TilemapSize {
                        x: game_data.map_size_x,
                        y: game_data.map_size_y,
                    },
                ) {
                    //println!("{:?}", tile_pos);
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
            let mut target_tile_pos = TilePos::default();
            if actions.target_world_pos {
                if let Some(tile_pos) = world_pos_to_tile_pos(
                    &cursor_world_pos.cursor_world_pos,
                    &TilemapSize {
                        x: game_data.map_size_x,
                        y: game_data.map_size_y,
                    },
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
