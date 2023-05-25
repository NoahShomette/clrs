use crate::game::{start_game, GameBuildSettings};
use crate::loading::level_loader::{LevelHandle, Levels};
use crate::loading::FontAssets;
use crate::ui::{modal_panel, BasicButton, DisabledButton, PlayerColors};
use crate::GameState;
use bevy::app::AppExit;
use bevy::prelude::CoreSet::Update;
use bevy::prelude::*;

pub struct PauseUiPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for PauseUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_menu.in_schedule(OnEnter(GameState::Paused)))
            .add_system(
                button_interaction
                    .in_base_set(Update)
                    .run_if(in_state(GameState::Paused)),
            )
            .add_system(cleanup_menu.in_schedule(OnExit(GameState::Paused)));
    }
}

#[derive(Component)]
struct PauseUiThing;

#[derive(Component)]
struct PauseUiCloseButton;

#[derive(Component)]
struct QuitButton;

#[derive(Component)]
struct MainMenuButton;

#[derive(Component)]
struct ContinueButton;

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    player_colors: Res<PlayerColors>,
) {
    let modal = modal_panel(
        PauseUiThing,
        true,
        Some(PauseUiCloseButton),
        &mut commands,
        &font_assets,
    );
    commands.entity(modal).with_children(|parent| {
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
            .insert(QuitButton)
            .insert(BasicButton)
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Quit to Desktop",
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
    mut exit: EventWriter<AppExit>,
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
            Option<&QuitButton>,
        ),
        (Changed<Interaction>, (With<Button>)),
    >,
    font_assets: Res<FontAssets>,
) {
    for (_, interaction, option_disabled, option_pucb, option_cb, option_mmb, option_qb) in
        &mut interaction_query
    {
        if Interaction::Clicked != *interaction {
            continue;
        }

        if let Some(_) = option_disabled {
            continue;
        }

        if let Some(_) = option_pucb {
            state.set(GameState::Playing);
        }
        if let Some(_) = option_cb {
            state.set(GameState::Playing);
        }
        if let Some(_) = option_mmb {
            state.set(GameState::Menu);
        }

        if let Some(_) = option_qb {
            exit.send(AppExit);
        }
    }
}

fn cleanup_menu(mut commands: Commands, button: Query<Entity, With<PauseUiThing>>) {
    for button in button.iter() {
        commands.entity(button).despawn_recursive();
    }
}
