use std::time::Duration;

use crate::audio::UiSoundEvents;
use crate::color_system::TileColor;
use crate::game::end_game::GameEnded;
use crate::game::restart_game::{RestartGame, RestartGameEvent};
use crate::game::{GameBuildSettings, GameData};
use crate::loading::FontAssets;
use crate::player::PlayerPoints;
use crate::ui::{modal_panel, BasicButton, DisabledButton, ModalStyle, PlayerColors};
use crate::GameState;
use bevy::prelude::CoreSet::Update;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::object::Object;
use bevy_ggf::player::{Player, PlayerMarker};
use bevy_tweening::{Animator, EaseFunction, RepeatCount, Tween};

use super::{UiBackgroundColorLens, UiMarginLens};

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
            .add_system(generate_all_player_cubes.run_if(in_state(GameState::Ended)))
            .add_system(cleanup_menu.in_schedule(OnExit(GameState::Ended)));
    }
}

#[derive(Component)]
struct EndGameUiThing;

#[derive(Component)]
struct MenuButton;

#[derive(Component)]
struct RestartButton;

#[derive(Component)]
struct PlayerCubesParent;

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    player_colors: Res<PlayerColors>,
    game_ended: Res<GameEnded>,
) {
    commands.init_resource::<CubeTimer>();

    let modal = modal_panel(
        EndGameUiThing,
        ModalStyle {
            with_close_button: false,
            close_button_bundle: None::<MenuButton>,
            modal_size: Some(Size::new(Val::Percent(60.0), Val::Percent(90.0))),
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

        parent.spawn(
            TextBundle::from_sections(vec![TextSection::new(
                "% Map Conquered",
                TextStyle {
                    font: font_assets.fira_sans.clone(),
                    font_size: 60.0,
                    color: Color::GRAY,
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
            .insert((PlayerCubesParent, EndGameUiThing));

        parent
            .spawn(NodeBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Relative,
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                background_color: Color::rgba(0.65, 0.65, 0.1, 0.0).into(),
                ..default()
            })
            .with_children(|parent| {
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
            });
    });
}

fn button_interaction(
    mut state: ResMut<NextState<GameState>>,
    mut restart_game: EventWriter<RestartGameEvent>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            Option<&DisabledButton>,
            Option<&MenuButton>,
            Option<&RestartButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (_, interaction, option_disabled, option_mb, option_rb) in &mut interaction_query {
        if Interaction::Clicked != *interaction {
            continue;
        }

        if let Some(_) = option_disabled {
            continue;
        }

        if let Some(_) = option_mb {
            state.set(GameState::Menu);
        }
        if let Some(_) = option_rb {
            restart_game.send(RestartGameEvent);
        }
    }
}

fn cleanup_menu(mut commands: Commands, button: Query<Entity, With<EndGameUiThing>>) {
    commands.remove_resource::<CubeTimer>();
    for button in button.iter() {
        commands.entity(button).despawn_recursive();
    }
}

#[derive(Resource)]
struct CubeTimer {
    pub timer: Timer,
    pub times_run: u8,
}

impl Default for CubeTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            times_run: Default::default(),
        }
    }
}

/// Component that marks which players cubes are held by this holder
#[derive(Component)]
struct PlayerIndex(usize);

fn generate_all_player_cubes(
    parent_query: Query<Entity, With<PlayerCubesParent>>,
    cube_holder: Query<(Entity, &PlayerIndex)>,
    mut menu_sound_events: EventWriter<UiSoundEvents>,
    player_colors: Res<PlayerColors>,
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
    mut commands: Commands,
    mut cube_timer: ResMut<CubeTimer>,
    time: Res<Time>,
) {
    cube_timer.timer.tick(time.delta());
    if !cube_timer.timer.just_finished() || cube_timer.times_run > 10 {
        return;
    }
    cube_timer.times_run += 1;
    let mut player_tile_count: HashMap<usize, i32> = HashMap::new();
    let Ok(parent) = parent_query.get_single() else {
        return;
    };

    for (_, _, _, option) in tile_queries.iter() {
        match option {
            None => {}
            Some((_, player_marker)) => {
                let count = player_tile_count.entry(player_marker.id()).or_insert(0);
                let count = *count;
                player_tile_count.insert(player_marker.id(), count.saturating_add(1));
            }
        }
    }

    let mut players: Vec<(&Player, &PlayerPoints)> = player_queries.iter().collect();
    players.sort_by(|a, b| a.0.id().cmp(&b.0.id()));

    let mut played_sound = false;

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

        let cube_parent = cube_holder
            .iter()
            .find(|(_, index)| index.0 == player_query.id());

        let cube_entity = match cube_parent {
            Some((entity, _)) => Some(entity),
            None => None,
        };

        if generate_player_cubes(
            parent,
            cube_entity,
            &mut commands,
            player_query.id(),
            player_tile_count
                .get(&player_query.id())
                .unwrap_or(&0)
                .clone() as usize,
            max_tile_count,
            &player_colors,
            cube_timer.times_run,
        ) {
            if !played_sound {
                menu_sound_events.send(UiSoundEvents::PlayerBoxAnimationEndGame);
                played_sound = true;
            }
        } else {
            if !played_sound {
                menu_sound_events.send(UiSoundEvents::PlayerBoxAnimationLostEndGame);
                played_sound = true;
            }
        }
    }
}

fn generate_player_cubes(
    parent: Entity,
    cube_holder: Option<Entity>,
    commands: &mut Commands,
    player_id: usize,
    player_tile_count: usize,
    max_tile_count: usize,
    player_colors: &Res<PlayerColors>,
    index: u8,
) -> bool {
    let mut played_sound = false;

    let cube_holder = match cube_holder {
        Some(entity) => entity,
        None => {
            let cube_holder = commands
                .spawn((
                    NodeBundle {
                        style: Style {
                            size: Size::new(Val::Auto, Val::Auto),
                            position_type: PositionType::Relative,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::ColumnReverse,
                            ..default()
                        },
                        background_color: Color::rgba(0.65, 0.65, 0.1, 0.0).into(),
                        ..default()
                    },
                    PlayerIndex(player_id),
                ))
                .id();
            commands.entity(parent).push_children(&[cube_holder]);
            cube_holder
        }
    };

    let player_tile_count = player_tile_count as f32 / max_tile_count as f32;

    let size = if index == 1 {
        Size::new(Val::Px(100.0), Val::Px(25.0))
    } else {
        Size::new(Val::Px(75.0), Val::Px(40.0))
    };

    let transform_tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(150),
        UiMarginLens {
            start: UiRect {
                left: Val::Px(5.0),
                top: Val::Px(30.0),
                right: Val::Px(5.0),
                bottom: Val::Px(5.0),
            },
            end: UiRect {
                left: Val::Px(5.0),
                top: Val::Px(5.0),
                right: Val::Px(5.0),
                bottom: Val::Px(5.0),
            },
        },
    )
    .with_repeat_count(RepeatCount::Finite(1));

    let mut color = Color::rgba(0.05, 0.05, 0.05, 1.0);

    if index as f32 / 10.0 < player_tile_count {
        color = player_colors.get_color(player_id);
        played_sound = true;
    };

    if index == 1 {
        color = player_colors.get_color(player_id);
    }

    let color_tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(200),
        UiBackgroundColorLens {
            start: Color::rgba(1.0, 1.0, 1.0, 0.0),
            end: color,
        },
    )
    .with_repeat_count(RepeatCount::Finite(1));

    let cube = commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: size,
                    position_type: PositionType::Relative,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                background_color: bevy::prelude::BackgroundColor(color),
                ..default()
            },
            Animator::new(transform_tween),
            Animator::new(color_tween),
        ))
        .id();

    commands.entity(cube_holder).push_children(&[cube]);
    played_sound
}
