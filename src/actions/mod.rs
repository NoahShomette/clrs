use crate::abilities::Abilities;
use crate::actions::game_control::{place_ability, place_building};
use bevy::prelude::*;
use bevy_ggf::game_core::saving::{BinaryComponentId, SaveId};
use bevy_ggf::mapping::tiles::TilePosition;
use bevy_ggf::player::PlayerMarker;
use serde::{Deserialize, Serialize};

use crate::buildings::BuildingTypes;
use crate::game::simulate_game;
use crate::{GamePausedState, GameState};

mod game_control;

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_actions.in_set(OnUpdate(GameState::Playing)));
        app.add_system(handle_pause);

        app.add_system(
            place_building
                .after(simulate_game)
                .in_schedule(CoreSchedule::Main)
                .run_if(in_state(GameState::Playing)),
        );
        app.add_system(
            place_ability
                .after(place_building)
                .in_schedule(CoreSchedule::Main)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Default, Component, Reflect, FromReflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Actions {
    pub try_place_building: bool,
    pub try_place_ability: bool,
    pub placed_building: bool,
    pub placed_ability: bool,
    pub selected_building: BuildingTypes,
    pub selected_ability: Abilities,
    pub target_world_pos: bool,
    pub building_tile_pos: Option<TilePosition>,
    pub ability_tile_pos: Option<TilePosition>,
}

impl SaveId for Actions {
    fn save_id(&self) -> BinaryComponentId {
        21
    }

    fn save_id_const() -> BinaryComponentId
    where
        Self: Sized,
    {
        21
    }

    #[doc = r" Serializes the state of the object at the given tick into binary. Only saves the keyframe and not the curve itself"]
    fn to_binary(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}

#[derive(Default, Resource)]
pub struct PauseGame;

#[derive(Default, Resource)]
pub struct UnPauseGame;

fn handle_pause(
    current_state: ResMut<State<GamePausedState>>,
    mut paused_next_state: ResMut<NextState<GamePausedState>>,
    mut commands: Commands,
    pause_game: Option<Res<PauseGame>>,
    unpause_game: Option<Res<UnPauseGame>>,
) {
    match current_state.0 {
        GamePausedState::NotPaused => {
            if pause_game.is_some() {
                paused_next_state.set(GamePausedState::Paused);
                commands.remove_resource::<PauseGame>();
            }
        }
        GamePausedState::Paused => {
            if unpause_game.is_some() {
                paused_next_state.set(GamePausedState::NotPaused);
                commands.remove_resource::<UnPauseGame>();
            }
        }
    }
}

pub fn update_actions(
    mouse: Res<Input<MouseButton>>,
    mut actions: Query<(&PlayerMarker, &mut Actions)>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    current_state: Res<State<GamePausedState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match current_state.0 {
            GamePausedState::NotPaused => commands.insert_resource(PauseGame),
            GamePausedState::Paused => commands.insert_resource(UnPauseGame),
        }
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

            if keyboard_input.just_pressed(KeyCode::Q) {
                actions.selected_building = BuildingTypes::Pulser
            }
            if keyboard_input.just_pressed(KeyCode::W) {
                actions.selected_building = BuildingTypes::Scatter
            }
            if keyboard_input.just_pressed(KeyCode::E) {
                actions.selected_building = BuildingTypes::Line
            }

            if keyboard_input.just_pressed(KeyCode::A) {
                actions.selected_ability = Abilities::Nuke;
            }
            if keyboard_input.just_pressed(KeyCode::S) {
                actions.selected_ability = Abilities::Fortify;
            }
            if keyboard_input.just_pressed(KeyCode::D) {
                actions.selected_ability = Abilities::Expand;
            }
        }
    }
}
