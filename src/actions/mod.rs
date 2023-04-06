use bevy::prelude::*;
use bevy_ascii_terminal::{StringFormatter, Terminal, TileFormatter, ToWorld};
use bevy_ecs_tilemap::prelude::TilePos;
use bevy_ggf::game_core::command::GameCommands;
use bevy_ggf::game_core::Game;
use bevy_ggf::mapping::tiles::ObjectStackingClass;
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::{Object, ObjectGridPosition, ObjectInfo};
use ns_defaults::camera::{ClickEvent, CursorWorldPos};
use num::clamp;

use crate::buildings::{Building, Pulser};
use crate::game::{GameData, BORDER_PADDING_TOTAL};
use crate::GameState;

mod game_control;

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Actions>()
            .add_system(set_movement_actions.in_set(OnUpdate(GameState::Playing)));
    }
}

#[derive(Default, Resource)]
pub struct Actions {
    pub player_movement: Option<Vec2>,
}

pub fn set_movement_actions(
    cursor_world_pos: Res<CursorWorldPos>,
    mouse: Res<Input<MouseButton>>,
    mut term_query: Query<(&mut Terminal, &ToWorld)>,
    mut game: ResMut<GameCommands>,
    game_data: Res<GameData>,
) {
    let (mut term, to_world) = term_query.single_mut();
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(world_pos) = to_world.screen_to_world(cursor_world_pos.cursor_world_pos) {
            let terminal_pos = to_world.world_to_tile(world_pos);


            println!("{:?}", terminal_pos);
            
           // println!("terminal_pos.x > (BORDER_PADDING_TOTAL / 2) as i32: {:?}", terminal_pos.x > (BORDER_PADDING_TOTAL / 2) as i32);
           // println!("terminal_pos.x < (game_data.map_size_x - (BORDER_PADDING_TOTAL / 2)) as i32: {:?}", terminal_pos.x.saturating_sub((BORDER_PADDING_TOTAL / 2) as i32) < (game_data.map_size_x) as i32);
           // println!("terminal_pos.y > (BORDER_PADDING_TOTAL / 2) as i32: {:?}", terminal_pos.y > (BORDER_PADDING_TOTAL / 2) as i32);
           // println!("terminal_pos.y.saturating_sub((BORDER_PADDING_TOTAL / 2) as i32) < (game_data.map_size_y) as i32: {:?}", terminal_pos.y.saturating_sub((BORDER_PADDING_TOTAL / 2) as i32) < (game_data.map_size_y) as i32);

            //term.in_bounds(terminal_pos) 
            if terminal_pos.x >= (BORDER_PADDING_TOTAL / 2) as i32
                    && terminal_pos.x.saturating_sub((BORDER_PADDING_TOTAL / 2) as i32) < (game_data.map_size_x) as i32
                    && terminal_pos.y >= (BORDER_PADDING_TOTAL / 2) as i32
                    && terminal_pos.y.saturating_sub((BORDER_PADDING_TOTAL / 2) as i32) < (game_data.map_size_y) as i32
            {
                println!("Got here");

                term.put_char(terminal_pos, 'X'.fg(Color::GREEN));

                /*
                let tile_pos: UVec2 = UVec2 {
                    x: clamp(terminal_pos.x as u32, 0, 30),
                    y: clamp(terminal_pos.y as u32, 0, 30),
                };
                 */
                let tile_pos: UVec2 = UVec2 {
                    x: terminal_pos.x.saturating_sub((BORDER_PADDING_TOTAL / 2) as i32) as u32,
                    y: terminal_pos.y.saturating_sub((BORDER_PADDING_TOTAL / 2) as i32) as u32,
                };
                
                let player_spawn_pos = TilePos {
                    x: tile_pos.x,
                    y: tile_pos.y,
                };

                let _ = game.spawn_object(
                    (
                        ObjectGridPosition {
                            tile_position: player_spawn_pos,
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
                            building_type: Pulser { strength: 5 },
                        },
                    ),
                    player_spawn_pos,
                    MapId { id: 1 },
                    0,
                );
            }
        }
    }
}
