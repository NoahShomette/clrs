use crate::loading::FontAssets;
use crate::ui::{modal_panel, BasicButton, DisabledButton, PlayerColors};
use crate::GameState;
use bevy::app::AppExit;
use bevy::prelude::CoreSet::Update;
use bevy::prelude::*;

pub struct EndGameUiPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for EndGameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_menu.in_schedule(OnEnter(GameState::Ended)))
            .add_system(
                button_interaction
                    .in_base_set(Update)
                    .run_if(in_state(GameState::Ended)),
            )
            .add_system(cleanup_menu.in_schedule(OnExit(GameState::Ended)));
    }
}

#[derive(Component)]
struct EndGameUiThing;

#[derive(Component)]
struct MenuButton;

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    player_colors: Res<PlayerColors>,
) {
    let modal = modal_panel(
        EndGameUiThing,
        false,
        None::<MenuButton>,
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
            .insert(EndGameUiThing)
            .insert(MenuButton)
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
    mut exit: EventWriter<AppExit>,
    mut player_colors: ResMut<PlayerColors>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            Option<&DisabledButton>,
            Option<&MenuButton>,
        ),
        (Changed<Interaction>, (With<Button>)),
    >,
    font_assets: Res<FontAssets>,
) {
    for (_, interaction, option_disabled, option_mb) in &mut interaction_query {
        if Interaction::Clicked != *interaction {
            continue;
        }

        if let Some(_) = option_disabled {
            continue;
        }

        if let Some(_) = option_mb {
            state.set(GameState::Menu);
        }
    }
}

fn cleanup_menu(mut commands: Commands, button: Query<Entity, With<EndGameUiThing>>) {
    for button in button.iter() {
        commands.entity(button).despawn_recursive();
    }
}
