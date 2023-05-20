use crate::game::GameBuildSettings;
use crate::level_loader::{LevelHandle, Levels};
use crate::GameState;
use bevy::app::AppExit;
use bevy::prelude::*;
use ns_defaults::camera::GGFCamera2dBundle;

use crate::ui::{MenuNavigation, PlayerColors};

pub struct MenuPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {}
}

pub fn setup_menu(mut commands: Commands) {}

pub fn handle_menu(
    mut next_state: ResMut<NextState<GameState>>,
    mut game_build_settings: ResMut<GameBuildSettings>,
    mut menu_nav: Local<MenuNavigation>,
    keyboard_input: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut player_colors: ResMut<PlayerColors>,
    mut levels: Res<LevelHandle>,
    mut assets: Res<Assets<Levels>>,
) {
    if keyboard_input.just_pressed(KeyCode::W) {
        menu_nav.0 = menu_nav.0.saturating_sub(1);
    }
    if keyboard_input.just_pressed(KeyCode::S) {
        menu_nav.0 = menu_nav.0.saturating_add(1);
        let max_nav = 4;

        #[cfg(not(target_arch = "wasm32"))]
        let max_nav = 5;

        if menu_nav.0 > max_nav {
            menu_nav.0 = max_nav;
        }
    }

    let modifier = keyboard_input.pressed(KeyCode::LShift);

    if menu_nav.0 == 0 && keyboard_input.just_pressed(KeyCode::Space)
        || keyboard_input.just_pressed(KeyCode::Insert)
    {
        next_state.set(GameState::Playing);
    }

    if menu_nav.0 == 1 {
        if keyboard_input.just_pressed(KeyCode::A) {
            game_build_settings.prev_map();
        }
        if keyboard_input.just_pressed(KeyCode::D) {
            game_build_settings.next_map();
        }
    }

    if menu_nav.0 == 2 && game_build_settings.map_type == 0 {
        if keyboard_input.just_pressed(KeyCode::A) {
            game_build_settings.decrease_map_size(modifier);
        }
        if keyboard_input.just_pressed(KeyCode::D) {
            game_build_settings.increase_map_size(modifier);
        }
    }

    if menu_nav.0 == 3 {
        if keyboard_input.just_pressed(KeyCode::A) {
            game_build_settings.decrease_enemy_count();
        }
        if keyboard_input.just_pressed(KeyCode::D) {
            game_build_settings.increase_enemy_count();
        }
    }

    if menu_nav.0 == 4 {
        if keyboard_input.just_pressed(KeyCode::A) {
            player_colors.prev_palette();
        }
        if keyboard_input.just_pressed(KeyCode::D) {
            player_colors.next_palette();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    if menu_nav.0 == 4 && keyboard_input.just_pressed(KeyCode::Space)
        || keyboard_input.just_pressed(KeyCode::Insert)
    {
        exit.send(AppExit);
    }
}
