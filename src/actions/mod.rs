use crate::abilities::Abilities;
use crate::actions::game_control::{place_building};
use bevy::prelude::*;
use bevy_ascii_terminal::TileFormatter;
use bevy_ggf::player::{Player, PlayerMarker};

use crate::buildings::BuildingTypes;
use crate::GameState;

mod game_control;

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_actions.in_set(OnUpdate(GameState::Playing)));
        app.add_system(
            place_building
                .in_set(OnUpdate(GameState::Playing))
                .after(update_actions),
        );
    }
}

#[derive(Default, Component, Reflect, FromReflect)]
#[reflect(Component)]
pub struct Actions {
    pub placed_building: bool,
    pub placed_ability: bool,
    pub selected_building: BuildingTypes,
    pub selected_ability: Abilities,
}

pub fn update_actions(
    mouse: Res<Input<MouseButton>>,
    mut actions: Query<(&PlayerMarker, &mut Actions)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    for (player, mut actions) in actions.iter_mut() {
        if player.id() == 0 {
            if mouse.just_pressed(MouseButton::Left) {
                actions.placed_building = true;
            }
            if mouse.just_pressed(MouseButton::Right) {
                actions.placed_ability = true;
            }

            if keyboard_input.just_pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
                match actions.selected_ability {
                    Abilities::Nuke => {
                        actions.selected_ability = Abilities::Sacrifice;
                    }
                    Abilities::Sacrifice => {
                        actions.selected_ability = Abilities::Boost;
                    }
                    Abilities::Boost => {}
                }
            }

            if keyboard_input.just_pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
                match actions.selected_ability {
                    Abilities::Nuke => {}
                    Abilities::Sacrifice => {
                        actions.selected_ability = Abilities::Nuke;
                    }
                    Abilities::Boost => {
                        actions.selected_ability = Abilities::Sacrifice;
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
