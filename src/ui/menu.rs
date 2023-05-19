use crate::color_system::PlayerColors;
use crate::game::GameBuildSettings;
use crate::level_loader::{LevelHandle, Levels};
use crate::GameState;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_ascii_terminal::{
    AutoCamera, Border, ColorFormatter, StringFormatter, Terminal, TerminalBundle, ToWorld,
};
use crate::ui::MenuNavigation;

pub struct MenuPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        
    }
}

pub fn setup_menu(mut commands: Commands, mut term_query: Query<&mut Terminal>) {
    if let Ok(mut term) = term_query.get_single_mut() {
        term.resize([30, 30]);
    } else {
        let term = Terminal::new([30, 30]).with_border(Border::single_line());
        commands
            .spawn((TerminalBundle::from(term), AutoCamera))
            .insert(ToWorld::default());
    }
}

pub fn handle_menu(
    mut next_state: ResMut<NextState<GameState>>,
    mut game_build_settings: ResMut<GameBuildSettings>,
    mut menu_nav: Local<MenuNavigation>,
    mut term_query: Query<&mut Terminal>,
    keyboard_input: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut player_colors: ResMut<PlayerColors>,
    mut levels: Res<LevelHandle>,
    mut assets: Res<Assets<Levels>>,
) {
    let mut term = term_query.single_mut();

    term.clear();

    term.put_string([1, 28], "Hello world!".fg(player_colors.get_color(0)));
    term.put_string([1, 19], "CLRS".fg(Color::WHITE));

    term.put_string([1, 18], "----".fg(Color::WHITE));

    term.put_string([1, 16], "PLAY".fg(Color::WHITE));

    term.put_string([0, 14], "<".fg(Color::WHITE));
    term.put_string([1, 14], "MAP".fg(Color::WHITE));
    term.put_string([4, 14], ">".fg(Color::WHITE));
    term.put_string(
        [1, 13],
        String::from(format!(
            "{}",
            assets.get(&levels.levels).unwrap().levels[game_build_settings.map_type].name
        ))
        .fg(Color::WHITE),
    );

    term.put_string([0, 11], "<".fg(Color::WHITE));
    term.put_string([1, 11], "SIZE".fg(Color::WHITE));
    term.put_string([5, 11], ">".fg(Color::WHITE));
    term.put_string(
        [1, 10],
        String::from(format!(
            "{}x{}",
            game_build_settings.map_size, game_build_settings.map_size
        ))
        .fg(Color::WHITE),
    );
    
    if game_build_settings.map_type != 0 {
        term.put_string([0, 11], "--------".fg(Color::WHITE));
        term.clear_string([1, 10], 10);
    }

    term.put_string([0, 8], "<".fg(Color::WHITE));
    term.put_string([1, 8], "ENEMIES".fg(Color::WHITE));
    term.put_string([8, 8], ">".fg(Color::WHITE));
    term.put_string(
        [1, 7],
        String::from(format!("{}", game_build_settings.enemy_count)).fg(Color::WHITE),
    );

    term.put_string([0, 5], "<".fg(Color::WHITE));
    term.put_string([1, 5], "CLRS".fg(Color::WHITE));
    term.put_string([6, 5], ">".fg(Color::WHITE));
    term.put_color([1, 4], player_colors.get_color(0).bg());
    term.put_color([2, 4], player_colors.get_color(1).bg());
    term.put_color([3, 4], player_colors.get_color(2).bg());
    term.put_color([4, 4], player_colors.get_color(3).bg());

    #[cfg(not(target_arch = "wasm32"))]
    term.put_string([1, 2], "QUIT".fg(Color::WHITE));

    if menu_nav.0 == 0 {
        term.put_string([1, 16], "PLAY".fg(player_colors.get_color(0)));
    }

    if menu_nav.0 == 1 {
        term.put_string([1, 14], "MAP".fg(player_colors.get_color(0)));
    }

    if menu_nav.0 == 2 {
        term.put_string([1, 11], "SIZE".fg(player_colors.get_color(0)));
    }
    if menu_nav.0 == 3 {
        term.put_string([1, 8], "ENEMIES".fg(player_colors.get_color(0)));
    }
    if menu_nav.0 == 4 {
        term.put_string([1, 5], "CLRS".fg(player_colors.get_color(0)));
    }
    #[cfg(not(target_arch = "wasm32"))]
    if menu_nav.0 == 5 {
        term.put_string([1, 2], "QUIT".fg(player_colors.get_color(0)));
    }

    term.put_string([10, 4], "|".fg(Color::WHITE));
    term.put_string([10, 5], "|".fg(Color::WHITE));
    term.put_string([10, 6], "|".fg(Color::WHITE));
    term.put_string([10, 7], "|".fg(Color::WHITE));
    term.put_string([10, 8], "|".fg(Color::WHITE));
    term.put_string([10, 9], "|".fg(Color::WHITE));
    term.put_string([10, 10], "|".fg(Color::WHITE));
    term.put_string([10, 11], "|".fg(Color::WHITE));
    term.put_string([10, 12], "|".fg(Color::WHITE));
    term.put_string([10, 13], "|".fg(Color::WHITE));
    term.put_string([10, 14], "|".fg(Color::WHITE));
    term.put_string([10, 15], "|".fg(Color::WHITE));
    term.put_string([10, 16], "|".fg(Color::WHITE));

    term.put_string([12, 18], "Buildings".fg(player_colors.get_color(0)));
    term.put_string([12, 16], "P -> Pulser".fg(Color::WHITE));
    term.put_string([12, 14], "S -> Scatter".fg(Color::WHITE));
    term.put_string([12, 12], "L -> Line".fg(Color::WHITE));

    term.put_string([12, 8], "Abilities".fg(player_colors.get_color(0)));
    term.put_string([12, 6], "N -> Nuke".fg(Color::WHITE));
    term.put_string([12, 4], "F -> Fortify".fg(Color::WHITE));
    term.put_string([12, 2], "E -> Expand".fg(Color::WHITE));

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
            term.put_string([0, 14], "<".fg(player_colors.get_color(0)));
        }
        if keyboard_input.just_pressed(KeyCode::D) {
            game_build_settings.next_map();
            term.put_string([4, 14], ">".fg(player_colors.get_color(0)));
        }
    }

    if menu_nav.0 == 2 && game_build_settings.map_type == 0 {
        if keyboard_input.just_pressed(KeyCode::A) {
            game_build_settings.decrease_map_size(modifier);
            term.put_string([0, 11], "<".fg(player_colors.get_color(0)));
        }
        if keyboard_input.just_pressed(KeyCode::D) {
            game_build_settings.increase_map_size(modifier);
            term.put_string([5, 11], ">".fg(player_colors.get_color(0)));
        }
    }

    if menu_nav.0 == 3 {
        if keyboard_input.just_pressed(KeyCode::A) {
            game_build_settings.decrease_enemy_count();
            term.put_string([0, 8], "<".fg(player_colors.get_color(0)));
        }
        if keyboard_input.just_pressed(KeyCode::D) {
            game_build_settings.increase_enemy_count();
            term.put_string([8, 8], ">".fg(player_colors.get_color(0)));
        }
    }

    if menu_nav.0 == 4 {
        if keyboard_input.just_pressed(KeyCode::A) {
            player_colors.prev_palette();
            term.put_string([0, 5], "<".fg(player_colors.get_color(0)));
        }
        if keyboard_input.just_pressed(KeyCode::D) {
            player_colors.next_palette();
            term.put_string([6, 5], ">".fg(player_colors.get_color(0)));
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    if menu_nav.0 == 4 && keyboard_input.just_pressed(KeyCode::Space)
        || keyboard_input.just_pressed(KeyCode::Insert)
    {
        exit.send(AppExit);
    }
}
