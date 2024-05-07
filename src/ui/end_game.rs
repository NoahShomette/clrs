use std::time::Duration;

use crate::actions::Actions;
use crate::color_system::TileColor;
use crate::game::end_game::GameEnded;
use crate::game::{GameBuildSettings, GameData};
use crate::loading::FontAssets;
use crate::player::PlayerPoints;
use crate::ui::{modal_panel, BasicButton, DisabledButton, ModalStyle, PlayerColors};
use crate::GameState;
use bevy::app::AppExit;
use bevy::prelude::CoreSet::Update;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::object::Object;
use bevy_ggf::player::{Player, PlayerMarker};
use bevy_tweening::lens::{TransformPositionLens, TransformScaleLens, UiPositionLens};
use bevy_tweening::{Animator, EaseFunction, RepeatCount, Tween};

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

#[derive(Component)]
struct PlayerCubesParent;

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    player_colors: Res<PlayerColors>,
    game_ended: Res<GameEnded>,
    tile_queries: Query<
        (
            &Tile,
            &TileTerrainInfo,
            &TilePos,
            Option<(&TileColor, &PlayerMarker)>,
        ),
        Without<Object>,
    >,
    player_queries: Query<(&Player, &PlayerPoints), Without<PlayerMarker>>,
    game: Res<GameData>,
    game_build_settings: Res<GameBuildSettings>,
) {
    let modal = modal_panel(
        EndGameUiThing,
        ModalStyle {
            with_close_button: false,
            close_button_bundle: None::<MenuButton>,
            modal_size: Some(Size::new(Val::Percent(60.0), Val::Percent(80.0))),
        },
        &mut commands,
        &font_assets,
    );
    commands.entity(modal).with_children(|parent| {
        let victory_text = match game_ended.player_won {
            true => "You Won!".to_string(),
            false => format!("You Lost to AI #{}!", game_ended.winning_id),
        };

        let winner_color = match game_ended.player_won {
            true => 0,
            false => game_ended.winning_id,
        };

        parent.spawn(
            TextBundle::from_sections(vec![TextSection::new(
                victory_text,
                TextStyle {
                    font: font_assets.fira_sans.clone(),
                    font_size: 100.0,
                    color: player_colors.get_color(winner_color),
                },
            )])
            .with_text_alignment(TextAlignment::Center)
            .with_style(Style {
                position_type: PositionType::Relative,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::vertical(Val::Px(25.0)),
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

        parent
            .spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(90.0), Val::Percent(80.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Relative,
                    flex_direction: FlexDirection::Row,
                    margin: UiRect::all(Val::Px(25.0)),
                    ..default()
                },
                background_color: Color::rgba(0.65, 0.65, 0.1, 0.0).into(),
                ..default()
            })
            .insert((PlayerCubesParent, EndGameUiThing))
            .with_children(|mut parent| {
                generate_all_player_cubes(
                    &mut parent,
                    &player_colors,
                    &tile_queries,
                    &player_queries,
                    &game,
                    &game_build_settings,
                );
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

fn generate_all_player_cubes(
    parent: &mut ChildBuilder,
    player_colors: &Res<PlayerColors>,
    tile_queries: &Query<
        (
            &Tile,
            &TileTerrainInfo,
            &TilePos,
            Option<(&TileColor, &PlayerMarker)>,
        ),
        Without<Object>,
    >,
    player_queries: &Query<(&Player, &PlayerPoints), Without<PlayerMarker>>,
    game: &Res<GameData>,
    game_build_settings: &GameBuildSettings,
) {
    let mut player_tile_count: HashMap<usize, i32> = HashMap::new();

    for (tile, tile_terrain_info, tile_pos, option) in tile_queries.iter() {
        match option {
            None => {}
            Some((tile_color_strength, player_marker)) => {
                let count = player_tile_count.entry(player_marker.id()).or_insert(0);
                let count = *count;
                player_tile_count.insert(player_marker.id(), count.saturating_add(1));
            }
        }
    }

    let mut players: Vec<(&Player, &PlayerPoints)> = player_queries.iter().collect();
    players.sort_by(|a, b| a.0.id().cmp(&b.0.id()));

    // for each player generate their cubes and add them plus get players points and display them
    for (player_query, player_points) in players.iter().rev() {
        let max_tile_count = match game_build_settings.game_end_conditions {
            crate::game::end_game::GameEndConditions::Domination => {
                (game.map_size_x * game.map_size_y) as usize
            }
            crate::game::end_game::GameEndConditions::Percentage { target_percentage } => {
                ((game.map_size_x * game.map_size_y) as f32 * target_percentage) as usize
            }
        };

        generate_player_cubes(
            parent,
            player_query.id(),
            player_tile_count
                .get(&player_query.id())
                .unwrap_or(&0)
                .clone() as usize,
            max_tile_count,
            &player_colors,
        );
    }
}

fn generate_player_cubes(
    parent: &mut ChildBuilder,
    player_id: usize,
    player_tile_count: usize,
    max_tile_count: usize,
    player_colors: &Res<PlayerColors>,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Auto, Val::Auto),
                position_type: PositionType::Relative,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                margin: UiRect::all(Val::Px(5.0)),
                padding: UiRect::bottom(Val::Px(35.0)),
                ..default()
            },
            background_color: Color::rgba(0.65, 0.65, 0.1, 0.0).into(),
            ..default()
        })
        .with_children(|parent| {
            let player_tile_count = player_tile_count as f32 / max_tile_count as f32;
            parent.spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(75.0), Val::Px(25.0)),
                    position_type: PositionType::Relative,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                background_color: BackgroundColor::from(player_colors.get_color(player_id)),
                ..default()
            });

            for index in 1..=10 {
                let transform_tween = Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_millis(1000),
                    UiPositionLens {
                        start: UiRect {
                            left: Val::Auto,
                            top: Val::Px(-10.0),
                            right: Val::Auto,
                            bottom: Val::Auto,
                        },
                        end: UiRect {
                            left: Val::Auto,
                            top: Val::Auto,
                            right: Val::Auto,
                            bottom: Val::Auto,
                        },
                    },
                )
                .with_repeat_count(RepeatCount::Finite(1));
                let mut color = Color::BLACK;

                if index as f32 / 10.0 < player_tile_count {
                    color = player_colors.get_color(player_id);
                };

                parent.spawn((
                    NodeBundle {
                        style: Style {
                            size: Size::new(Val::Px(75.0), Val::Px(40.0)),
                            position_type: PositionType::Relative,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        background_color: BackgroundColor::from(color),
                        ..default()
                    },
                    Animator::new(transform_tween),
                ));
            }
        });
}
