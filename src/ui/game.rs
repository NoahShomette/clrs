use crate::game::GameBuildSettings;
use crate::level_loader::{LevelHandle, Levels};
use crate::loading::FontAssets;
use crate::GameState;
use bevy::app::AppExit;
use bevy::prelude::*;

use crate::ui::{MenuNavigation, PlayerColors};

pub struct GameUiPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_menu.in_schedule(OnEnter(GameState::Playing)))
            .add_system(button_interaction.in_set(OnUpdate(GameState::Playing)))
            .add_system(cleanup_menu.in_schedule(OnExit(GameState::Playing)));
    }
}

#[derive(Component)]
struct GameUiThing;

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    player_colors: Res<PlayerColors>,
) {
    commands
        .spawn(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(120.0), Val::Px(50.0)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: BackgroundColor::from(Color::GRAY),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Play",
                TextStyle {
                    font: font_assets.abaddon_bold.clone(),
                    font_size: 40.0,
                    color: player_colors.get_color(0),
                },
            ));
        });
}

fn button_interaction(
    mut state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&Interaction),
        (Changed<Interaction>, (With<Button>, With<GameUiThing>)),
    >,
) {
    for (interaction) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                state.set(GameState::Playing);
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

fn cleanup_menu(mut commands: Commands, button: Query<Entity, With<Button>>) {
    commands.entity(button.single()).despawn_recursive();
}
