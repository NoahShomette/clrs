use crate::abilities::Abilities;
use crate::actions::game_control::{place_ability, place_building};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilePos;
use bevy_ggf::game_core::Game;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::object::Object;
use bevy_ggf::player::{Player, PlayerMarker};

use crate::buildings::BuildingTypes;
use crate::game::{simulate_game, GameBuildSettings, GameData, BORDER_PADDING_TOTAL};
use crate::ui::{MenuNavigation, PlayerColors};
use crate::GameState;

mod game_control;

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_actions.in_set(OnUpdate(GameState::Playing)));
        app.add_system(paused_controls.in_set(OnUpdate(GameState::Paused)));
        app.add_system(ended_controls.in_set(OnUpdate(GameState::Ended)));
        app.add_system(handle_pause);

        app.add_system(
            place_building
                .after(simulate_game)
                .in_schedule(CoreSchedule::FixedUpdate)
                .run_if(in_state(GameState::Playing)),
        );
        app.add_system(
            place_ability
                .after(place_building)
                .in_schedule(CoreSchedule::FixedUpdate)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Default, Component, Reflect, FromReflect)]
#[reflect(Component)]
pub struct Actions {
    pub try_place_building: bool,
    pub try_place_ability: bool,
    pub placed_building: bool,
    pub placed_ability: bool,
    pub selected_building: BuildingTypes,
    pub selected_ability: Abilities,
    pub target_world_pos: bool,
    pub building_tile_pos: Option<TilePos>,
    pub ability_tile_pos: Option<TilePos>,
}

#[derive(Default, Resource)]
pub struct PauseGame;

#[derive(Default, Resource)]
pub struct UnPauseGame;

fn handle_pause(
    mut current_state: ResMut<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    pause_game: Option<Res<PauseGame>>,
    unpause_game: Option<Res<UnPauseGame>>,
) {
    match current_state.0 {
        GameState::Loading => {}
        GameState::Playing => {
            if let Some(pause_game) = pause_game {
                next_state.set(GameState::Paused);
                commands.remove_resource::<PauseGame>();
            }
        }
        GameState::Paused => {
            if let Some(unpause_game) = unpause_game {
                next_state.set(GameState::Playing);
                commands.remove_resource::<UnPauseGame>();
            }
        }
        GameState::Menu => {}
        GameState::Ended => {}
    }
}

pub fn paused_controls(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    game: Res<GameData>,
    mut menu_nav: Local<MenuNavigation>,
    mut next_state: ResMut<NextState<GameState>>,
    tiles: Query<Entity, With<Tile>>,
    objects: Query<Entity, With<Object>>,
    players: Query<Entity, With<Player>>,
    player_marker: Query<Entity, With<PlayerMarker>>,
    player_colors: Res<PlayerColors>,
) {
    /*
    let mut term = term_query.single_mut();
    let term_size = term.size();

    term.put_string([0, term_size.y - 3], "PLAY".fg(Color::WHITE));
    term.put_string([0, term_size.y - 5], "MENU".fg(Color::WHITE));

    term.put_string(
        [
            (term_size.x / 2) - (BORDER_PADDING_TOTAL / 2),
            game.map_size_y + (BORDER_PADDING_TOTAL / 2) + 6,
        ],
        "!!! PAUSED !!!".fg(player_colors.get_color(0)),
    );
    let max_nav = 2;

    if menu_nav.0 == 0 {
        term.put_string([0, term_size.y - 3], "PLAY".fg(player_colors.get_color(0)));
    }
    if menu_nav.0 == 1 {
        term.put_string([0, term_size.y - 5], "MENU".fg(player_colors.get_color(0)));
    }

     */

    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.insert_resource(UnPauseGame);
    }

    if keyboard_input.just_pressed(KeyCode::W) {
        menu_nav.0 = menu_nav.0.saturating_sub(1);
    }
    if keyboard_input.just_pressed(KeyCode::S) {
        menu_nav.0 = menu_nav.0.saturating_add(1);
        let max_nav = 1;

        if menu_nav.0 > max_nav {
            menu_nav.0 = max_nav;
        }
    }

    if menu_nav.0 == 0 && keyboard_input.just_pressed(KeyCode::Space)
        || keyboard_input.just_pressed(KeyCode::Insert)
    {
        next_state.set(GameState::Playing);
    }

    if menu_nav.0 == 1 && keyboard_input.just_pressed(KeyCode::Space)
        || keyboard_input.just_pressed(KeyCode::Insert)
    {
        next_state.set(GameState::Menu);

        for entity in tiles.iter() {
            commands.entity(entity).despawn();
        }
        for entity in objects.iter() {
            commands.entity(entity).despawn();
        }
        for entity in players.iter() {
            commands.entity(entity).despawn();
        }
        for entity in player_marker.iter() {
            commands.entity(entity).despawn();
        }

        commands.remove_resource::<Game>();
        commands.init_resource::<GameBuildSettings>();
    }
}

pub fn ended_controls(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<Input<KeyCode>>,
    tiles: Query<Entity, With<Tile>>,
    objects: Query<Entity, With<Object>>,
    players: Query<Entity, With<Player>>,
    player_marker: Query<Entity, With<PlayerMarker>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Menu);

        for entity in tiles.iter() {
            commands.entity(entity).despawn();
        }
        for entity in objects.iter() {
            commands.entity(entity).despawn();
        }
        for entity in players.iter() {
            commands.entity(entity).despawn();
        }
        for entity in player_marker.iter() {
            commands.entity(entity).despawn();
        }

        commands.remove_resource::<Game>();
        commands.init_resource::<GameBuildSettings>();
    }
}

pub fn update_actions(
    mouse: Res<Input<MouseButton>>,
    mut actions: Query<(&PlayerMarker, &mut Actions)>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.insert_resource(PauseGame);
    }

    for (player, mut actions) in actions.iter_mut() {
        actions.building_tile_pos = None;
        actions.target_world_pos = false;
        actions.try_place_ability = false;
        actions.try_place_building = false;
        actions.placed_building = false;
        actions.placed_ability = false;

        if player.id() == 0 {
            if mouse.just_pressed(MouseButton::Left) {
                actions.try_place_building = true;
                actions.target_world_pos = true;
            }
            if mouse.just_pressed(MouseButton::Right) {
                actions.try_place_ability = true;
                actions.target_world_pos = true;
            }

            if keyboard_input.just_pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
                match actions.selected_ability {
                    Abilities::Nuke => {
                        actions.selected_ability = Abilities::Fortify;
                    }
                    Abilities::Fortify => {
                        actions.selected_ability = Abilities::Expand;
                    }
                    Abilities::Expand => {}
                }
            }

            if keyboard_input.just_pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
                match actions.selected_ability {
                    Abilities::Nuke => {}
                    Abilities::Fortify => {
                        actions.selected_ability = Abilities::Nuke;
                    }
                    Abilities::Expand => {
                        actions.selected_ability = Abilities::Fortify;
                    }
                }
            }

            if keyboard_input.just_pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
                match actions.selected_building {
                    BuildingTypes::Pulser => {}
                    BuildingTypes::Scatter => actions.selected_building = BuildingTypes::Pulser,
                    BuildingTypes::Line => actions.selected_building = BuildingTypes::Scatter,
                }
            }

            if keyboard_input.just_pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
                match actions.selected_building {
                    BuildingTypes::Pulser => actions.selected_building = BuildingTypes::Scatter,
                    BuildingTypes::Scatter => actions.selected_building = BuildingTypes::Line,
                    BuildingTypes::Line => {}
                }
            }
        }
    }
}
