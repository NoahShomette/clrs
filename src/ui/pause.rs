use crate::audio::GameSoundSettings;
use crate::game::restart_game::{RestartGame, RestartGameEvent};
use crate::game::{start_game, GameBuildSettings};
use crate::loading::level_loader::{LevelHandle, Levels};
use crate::loading::FontAssets;
use crate::ui::{modal_panel, BasicButton, DisabledButton, ModalStyle, PlayerColors};
use crate::{GamePausedState, GameState};
use bevy::app::AppExit;
use bevy::prelude::CoreSet::Update;
use bevy::prelude::*;

use super::settings_menu::spawn_settings_menu;

pub struct PauseUiPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for PauseUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_menu.in_schedule(OnEnter(GamePausedState::Paused)))
            .add_system(
                button_interaction
                    .in_base_set(Update)
                    .run_if(in_state(GamePausedState::Paused)),
            )
            .add_system(cleanup_menu.in_schedule(OnExit(GamePausedState::Paused)));
    }
}

#[derive(Component, Clone)]
struct PauseUiThing;

#[derive(Component)]
struct PauseUiCloseButton;

#[derive(Component)]
struct SettingsButton;

#[derive(Component)]
struct MainMenuButton;

#[derive(Component)]
struct ContinueButton;

#[derive(Component)]
struct RestartButton;

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    player_colors: Res<PlayerColors>,
) {
    let modal = modal_panel(
        PauseUiThing,
        ModalStyle {
            with_close_button: false,
            close_button_bundle: Some(PauseUiCloseButton),
            modal_size: None,
        },
        &mut commands,
        &font_assets,
    );
    commands.entity(modal).with_children(|parent| {
        parent.spawn(
            TextBundle::from_sections(vec![
                TextSection::new(
                    "C",
                    TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 100.0,
                        color: player_colors.get_color(0),
                    },
                ),
                TextSection::new(
                    "L",
                    TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 100.0,
                        color: player_colors.get_color(1),
                    },
                ),
                TextSection::new(
                    "R",
                    TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 100.0,
                        color: player_colors.get_color(2),
                    },
                ),
                TextSection::new(
                    "S",
                    TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 100.0,
                        color: player_colors.get_color(3),
                    },
                ),
            ])
            .with_text_alignment(TextAlignment::Center)
            .with_style(Style {
                position_type: PositionType::Relative,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(0.0)),
                size: Size::new(Val::Auto, Val::Auto),
                ..default()
            }),
        );

        parent.spawn(
            TextBundle::from_sections(vec![TextSection::new(
                "Paused",
                TextStyle {
                    font: font_assets.fira_sans.clone(),
                    font_size: 50.0,
                    color: player_colors.get_color(0),
                },
            )])
            .with_text_alignment(TextAlignment::Center)
            .with_style(Style {
                position_type: PositionType::Relative,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(25.0)),
                size: Size::new(Val::Auto, Val::Auto),
                ..default()
            }),
        );

        parent
            .spawn(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Auto, Val::Px(50.0)),
                    margin: UiRect::all(Val::Px(10.0)),
                    padding: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: BackgroundColor::from(Color::GRAY),
                ..Default::default()
            })
            .insert(PauseUiThing)
            .insert(ContinueButton)
            .insert(BasicButton)
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Continue",
                    TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                ));
            });

        parent
            .spawn(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Auto, Val::Px(50.0)),
                    margin: UiRect::all(Val::Px(10.0)),
                    padding: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: BackgroundColor::from(Color::GRAY),
                ..Default::default()
            })
            .insert(PauseUiThing)
            .insert(SettingsButton)
            .insert(BasicButton)
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Settings",
                    TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                ));
            });

        parent
            .spawn(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Auto, Val::Px(50.0)),
                    margin: UiRect::all(Val::Px(10.0)),
                    padding: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: BackgroundColor::from(Color::GRAY),
                ..Default::default()
            })
            .insert(PauseUiThing)
            .insert(RestartButton)
            .insert(BasicButton)
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Restart",
                    TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                ));
            });

        parent
            .spawn(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Auto, Val::Px(50.0)),
                    margin: UiRect::all(Val::Px(10.0)),
                    padding: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: BackgroundColor::from(Color::GRAY),
                ..Default::default()
            })
            .insert(PauseUiThing)
            .insert(MainMenuButton)
            .insert(BasicButton)
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Main Menu",
                    TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                ));
            });
    });
}

fn button_interaction(
    mut state: ResMut<NextState<GameState>>,
    mut paused_state: ResMut<NextState<GamePausedState>>,
    mut restart_game: EventWriter<RestartGameEvent>,
    mut player_colors: ResMut<PlayerColors>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            Option<&DisabledButton>,
            Option<&PauseUiCloseButton>,
            Option<&ContinueButton>,
            Option<&MainMenuButton>,
            Option<&SettingsButton>,
            Option<&RestartButton>,
        ),
        (Changed<Interaction>, (With<Button>)),
    >,
    font_assets: Res<FontAssets>,
    sound_settings: Res<GameSoundSettings>,
) {
    for (
        _,
        interaction,
        option_disabled,
        option_pucb,
        option_cb,
        option_mmb,
        option_sb,
        option_rb,
    ) in &mut interaction_query
    {
        if Interaction::Clicked != *interaction {
            continue;
        }

        if option_disabled.is_some() {
            continue;
        }

        if option_pucb.is_some() {
            paused_state.set(GamePausedState::NotPaused);
        }
        if option_cb.is_some() {
            paused_state.set(GamePausedState::NotPaused);
        }
        if option_mmb.is_some() {
            state.set(GameState::Menu);
            paused_state.set(GamePausedState::NotPaused);
        }

        if let Some(_) = option_rb {
            restart_game.send(RestartGameEvent);
        }

        if option_sb.is_some() {
            spawn_settings_menu(
                PauseUiThing,
                ModalStyle {
                    with_close_button: true,
                    close_button_bundle: None::<SettingsButton>,
                    modal_size: None,
                },
                &mut commands,
                &font_assets,
                &player_colors,
                &sound_settings,
            );
        }
    }
}

fn cleanup_menu(mut commands: Commands, button: Query<Entity, With<PauseUiThing>>) {
    for button in button.iter() {
        commands.entity(button).despawn_recursive();
    }
}
