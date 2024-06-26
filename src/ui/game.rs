﻿use crate::abilities::Abilities;
use crate::actions::Actions;
use crate::buildings::BuildingTypes::{self, Line, Pulser, Scatter};
use crate::color_system::TileColor;
use crate::game::{start_game, GameBuildSettings, GameData};
use crate::loading::{FontAssets, TextureAssets};
use crate::player::PlayerPoints;
use crate::{GamePausedState, GameState};
use bevy::prelude::CoreSet::Update;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::TilePos;
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::object::Object;
use bevy_ggf::player::{Player, PlayerMarker};
use bevy_tweening::lens::TransformScaleLens;
use bevy_tweening::{Animator, EaseFunction, RepeatCount, RepeatStrategy, Tween};
use std::ops::DerefMut;
use std::time::Duration;

use crate::ui::{
    BasicButton, DisabledButton, GameButton, GameButtonIcon, PlayerColors, SelectedButton,
};

pub struct GameUiPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            setup_menu
                .after(start_game)
                .in_schedule(OnEnter(GameState::Playing)),
        )
        .add_system(
            button_interaction
                .in_base_set(Update)
                .run_if(in_state(GameState::Playing)),
        )
        .add_system(cleanup_menu.in_schedule(OnExit(GameState::Playing)));

        app.add_systems(
            (
                update_game_buttons_from_keyboard_selectiron,
                apply_system_buffers,
                handle_button_visuals,
            )
                .in_base_set(Update)
                .distributive_run_if(in_state(GameState::Playing)),
        );
        app.add_system(
            handle_selected_button
                .in_base_set(Update)
                .run_if(in_state(GameState::Playing)),
        );
        app.add_system(
            handle_player_cubes_and_stats
                .in_base_set(Update)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn update_game_buttons_from_keyboard_selectiron(
    mut interaction_query: Query<
        (
            &mut Interaction,
            Option<&LineButtonMarker>,
            Option<&PulserButtonMarker>,
            Option<&ScatterButtonMarker>,
            Option<&NukeButtonMarker>,
            Option<&ExpandButtonMarker>,
            Option<&FortifyButtonMarker>,
        ),
        (With<Button>, Without<DisabledButton>, With<GameButton>),
    >,
    actions: Query<(&PlayerMarker, &Actions), Changed<Actions>>,
    mut last_action: Local<Actions>,
) {
    for (player, actions) in actions.iter() {
        if player.id() == 0 {
            let mut selected_building_changed = false;
            let mut selected_ability_changed = false;

            if actions.selected_building != last_action.selected_building {
                selected_building_changed = true;
                last_action.selected_building = actions.selected_building;
            }

            if actions.selected_ability != last_action.selected_ability {
                selected_ability_changed = true;
                last_action.selected_ability = actions.selected_ability;
            }

            for (mut interaction, lbm, pbm, sbm, nbm, ebm, fbm) in interaction_query.iter_mut() {
                if selected_building_changed {
                    if lbm.is_some() && actions.selected_building == BuildingTypes::Line {
                        *interaction = Interaction::Clicked;
                    }
                    if pbm.is_some() && actions.selected_building == BuildingTypes::Pulser {
                        *interaction = Interaction::Clicked;
                    }
                    if sbm.is_some() && actions.selected_building == BuildingTypes::Scatter {
                        *interaction = Interaction::Clicked;
                    }
                }

                if selected_ability_changed {
                    if nbm.is_some() && actions.selected_ability == Abilities::Nuke {
                        *interaction = Interaction::Clicked;
                    }
                    if ebm.is_some() && actions.selected_ability == Abilities::Expand {
                        *interaction = Interaction::Clicked;
                    }
                    if fbm.is_some() && actions.selected_ability == Abilities::Fortify {
                        *interaction = Interaction::Clicked;
                    }
                }
            }
        }
    }
}

fn handle_button_visuals(
    player_colors: Option<Res<PlayerColors>>,
    mut interaction_query: Query<
        (Entity, &Interaction, &Children, Option<&SelectedButton>),
        (
            Changed<Interaction>,
            With<Button>,
            Without<DisabledButton>,
            With<GameButton>,
        ),
    >,
    mut background_color_query: Query<
        (&mut BackgroundColor, &Children),
        (Without<GameButtonIcon>, With<MainButtonBG>),
    >,
    mut children_text_color: Query<&mut Text, With<MainButtonText>>,
    mut commands: Commands,
) {
    if let Some(player_colors) = player_colors {
        for (entity, interaction, children, option_sb) in &mut interaction_query {
            match *interaction {
                Interaction::Clicked => {
                    let transform_tween = Tween::new(
                        EaseFunction::QuadraticInOut,
                        Duration::from_millis(50),
                        TransformScaleLens {
                            start: Vec3 {
                                x: 1.0,
                                y: 1.0,
                                z: 1.0,
                            },

                            end: Vec3 {
                                x: 1.4,
                                y: 1.0,
                                z: 1.0,
                            },
                        },
                    )
                    .with_repeat_count(RepeatCount::Finite(2))
                    .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

                    commands
                        .entity(entity)
                        .insert(Animator::new(transform_tween));
                }
                Interaction::Hovered => {
                    let transform_tween = Tween::new(
                        EaseFunction::QuadraticInOut,
                        Duration::from_millis(100),
                        TransformScaleLens {
                            start: Vec3 {
                                x: 1.0,
                                y: 1.0,
                                z: 1.0,
                            },

                            end: Vec3 {
                                x: 1.1,
                                y: 1.1,
                                z: 1.0,
                            },
                        },
                    )
                    .with_repeat_count(RepeatCount::Finite(2))
                    .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

                    commands
                        .entity(entity)
                        .insert(Animator::new(transform_tween));
                    for &child in children.iter() {
                        if let Ok((mut color, children)) = background_color_query.get_mut(child) {
                            *color = BackgroundColor::from(player_colors.get_color(0));
                            for &child in children.iter() {
                                if let Ok(mut text) = children_text_color.get_mut(child) {
                                    text.sections[0].style.color = Color::BLACK;
                                }
                            }
                        }
                    }
                }
                Interaction::None => {
                    for &child in children.iter() {
                        if let Ok((mut color, children)) = background_color_query.get_mut(child) {
                            if let Some(_) = option_sb {
                                *color = BackgroundColor::from(player_colors.get_color(0));
                            } else {
                                *color = BackgroundColor::from(Color::GRAY);
                            }
                            for &child in children.iter() {
                                if let Ok(mut text) = children_text_color.get_mut(child) {
                                    text.sections[0].style.color = Color::BLACK;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component)]
struct GameUiThing;

#[derive(Component)]
struct MainButtonText;

#[derive(Component)]
struct MainButtonBG;

#[derive(Component)]
struct PauseButton;

#[derive(Component)]
struct BuildingButtonsGroupMarker;

#[derive(Component)]
struct AbilitiesButtonsGroupMarker;

#[derive(Component)]
struct NewSelectedButton;

#[derive(Component)]
struct PulserButtonMarker;

#[derive(Component)]
struct LineButtonMarker;

#[derive(Component)]
struct ScatterButtonMarker;

#[derive(Component)]
struct NukeButtonMarker;

#[derive(Component)]
struct FortifyButtonMarker;

#[derive(Component)]
struct ExpandButtonMarker;

#[derive(Component)]
struct PlayerCubesParent;

#[derive(Component)]
struct PlayerCubesHolder;

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    player_colors: Res<PlayerColors>,
    texture_assets: Res<TextureAssets>,
    player_actions: Query<(&PlayerMarker, &Actions), Without<Player>>,
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
    for (player, actions) in player_actions.iter() {
        if player.id() != 0 {
            continue;
        }
        //root node for the entire main menu
        commands
            .spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            })
            .insert(GameUiThing)
            .with_children(|parent| {
                //root node for the main controls wrapping the entire control section on the left side
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(20.0), Val::Percent(100.0)),
                            justify_content: JustifyContent::SpaceEvenly,
                            align_items: AlignItems::Center,
                            position_type: PositionType::Relative,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        background_color: Color::rgba(0.65, 0.65, 0.65, 0.0).into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn(
                            TextBundle::from_section(
                                "CLRS",
                                TextStyle {
                                    font: font_assets.fira_sans.clone(),
                                    font_size: 100.0,
                                    color: player_colors.get_color(0),
                                },
                            )
                            .with_text_alignment(TextAlignment::Center)
                            .with_style(Style {
                                position_type: PositionType::Relative,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(75.0)),
                                size: Size::new(Val::Auto, Val::Auto),
                                ..default()
                            }),
                        );

                        // node wrapping the actual buttons
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    size: Size::new(Val::Percent(90.0), Val::Percent(80.0)),
                                    justify_content: JustifyContent::End,
                                    align_items: AlignItems::Center,
                                    position_type: PositionType::Relative,
                                    flex_direction: FlexDirection::Column,
                                    margin: UiRect::all(Val::Px(50.0)),
                                    ..default()
                                },
                                background_color: Color::rgba(0.65, 0.65, 0.1, 0.0).into(),
                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn(
                                    TextBundle::from_section(
                                        "BUILDINGS",
                                        TextStyle {
                                            font: font_assets.fira_sans.clone(),
                                            font_size: 40.0,
                                            color: player_colors.get_color(0),
                                        },
                                    )
                                    .with_text_alignment(TextAlignment::Center)
                                    .with_style(Style {
                                        position_type: PositionType::Relative,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::top(Val::Px(50.0)),
                                        size: Size::new(Val::Auto, Val::Auto),
                                        ..default()
                                    }),
                                );

                                parent.spawn(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(80.0), Val::Px(5.0)),
                                        position_type: PositionType::Relative,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::all(Val::Px(25.0)),
                                        ..default()
                                    },
                                    background_color: Color::DARK_GRAY.into(),
                                    ..default()
                                });

                                let mut pulsor_button = None::<NewSelectedButton>;
                                let mut scatter_button = None::<NewSelectedButton>;
                                let mut line_button = None::<NewSelectedButton>;

                                match actions.selected_building {
                                    Pulser => pulsor_button = Some(NewSelectedButton),
                                    Scatter => scatter_button = Some(NewSelectedButton),
                                    Line => line_button = Some(NewSelectedButton),
                                }

                                game_button(
                                    parent,
                                    &font_assets,
                                    GameUiThing,
                                    (PulserButtonMarker, BuildingButtonsGroupMarker),
                                    "Pulser",
                                    texture_assets.pulser.clone(),
                                    50,
                                    pulsor_button,
                                );

                                game_button(
                                    parent,
                                    &font_assets,
                                    GameUiThing,
                                    (ScatterButtonMarker, BuildingButtonsGroupMarker),
                                    "Scatter",
                                    texture_assets.scatter.clone(),
                                    50,
                                    scatter_button,
                                );
                                game_button(
                                    parent,
                                    &font_assets,
                                    GameUiThing,
                                    (LineButtonMarker, BuildingButtonsGroupMarker),
                                    "Line",
                                    texture_assets.line.clone(),
                                    50,
                                    line_button,
                                );

                                parent.spawn(
                                    TextBundle::from_section(
                                        "ABILITIES",
                                        TextStyle {
                                            font: font_assets.fira_sans.clone(),
                                            font_size: 40.0,
                                            color: player_colors.get_color(0),
                                        },
                                    )
                                    .with_text_alignment(TextAlignment::Center)
                                    .with_style(Style {
                                        position_type: PositionType::Relative,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::top(Val::Px(50.0)),
                                        size: Size::new(Val::Auto, Val::Auto),
                                        ..default()
                                    }),
                                );

                                parent.spawn(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(80.0), Val::Px(5.0)),
                                        position_type: PositionType::Relative,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::all(Val::Px(25.0)),
                                        ..default()
                                    },
                                    background_color: Color::DARK_GRAY.into(),
                                    ..default()
                                });

                                let mut nuke_button = None::<NewSelectedButton>;
                                let mut fortify_button = None::<NewSelectedButton>;
                                let mut expand_button = None::<NewSelectedButton>;

                                match actions.selected_ability {
                                    Abilities::Nuke => nuke_button = Some(NewSelectedButton),
                                    Abilities::Fortify => fortify_button = Some(NewSelectedButton),
                                    Abilities::Expand => expand_button = Some(NewSelectedButton),
                                }

                                game_button(
                                    parent,
                                    &font_assets,
                                    GameUiThing,
                                    (NukeButtonMarker, AbilitiesButtonsGroupMarker),
                                    "Nuke",
                                    texture_assets.nuke.clone(),
                                    50,
                                    nuke_button,
                                );

                                game_button(
                                    parent,
                                    &font_assets,
                                    GameUiThing,
                                    (FortifyButtonMarker, AbilitiesButtonsGroupMarker),
                                    "Fortify",
                                    texture_assets.fortify.clone(),
                                    50,
                                    fortify_button,
                                );
                                game_button(
                                    parent,
                                    &font_assets,
                                    GameUiThing,
                                    (ExpandButtonMarker, AbilitiesButtonsGroupMarker),
                                    "Expand",
                                    texture_assets.expand.clone(),
                                    50,
                                    expand_button,
                                );
                            });
                    });

                // Node wrapping entire right side ui
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(20.0), Val::Percent(100.0)),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            position_type: PositionType::Relative,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        background_color: Color::rgba(0.65, 0.65, 0.65, 0.0).into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn(ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Auto, Val::Px(50.0)),
                                    margin: UiRect::all(Val::Px(25.0)),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    position_type: PositionType::Relative,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                },
                                background_color: BackgroundColor::from(Color::GRAY),
                                ..Default::default()
                            })
                            .insert(BasicButton)
                            .insert(PauseButton)
                            .insert(GameUiThing)
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "PAUSE",
                                    TextStyle {
                                        font: font_assets.fira_sans.clone(),
                                        font_size: 40.0,
                                        color: Color::BLACK,
                                    },
                                ));
                            });

                        // node wrapping the actual buttons
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    size: Size::new(Val::Percent(90.0), Val::Percent(80.0)),
                                    justify_content: JustifyContent::Start,
                                    align_items: AlignItems::Center,
                                    position_type: PositionType::Relative,
                                    flex_direction: FlexDirection::Column,
                                    margin: UiRect::all(Val::Px(25.0)),
                                    ..default()
                                },
                                background_color: Color::rgba(0.65, 0.65, 0.1, 0.0).into(),
                                ..default()
                            })
                            .insert((PlayerCubesParent, GameUiThing))
                            .with_children(|mut parent| {
                                generate_all_player_cubes(
                                    &mut parent,
                                    &player_colors,
                                    &tile_queries,
                                    &player_queries,
                                    &font_assets,
                                    &game,
                                    &game_build_settings,
                                );
                            });
                    });
            });
    }
}

fn button_interaction(
    mut state: ResMut<NextState<GamePausedState>>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            Option<&DisabledButton>,
            Option<&PulserButtonMarker>,
            Option<&ScatterButtonMarker>,
            Option<&LineButtonMarker>,
            Option<&NukeButtonMarker>,
            Option<&FortifyButtonMarker>,
            Option<&ExpandButtonMarker>,
            Option<&PauseButton>,
        ),
        (Changed<Interaction>, (With<Button>, With<GameUiThing>)),
    >,
    mut actions: Query<(&PlayerMarker, &mut Actions)>,
    mut commands: Commands,
) {
    for (
        entity,
        interaction,
        option_disabled,
        option_pbm,
        option_sbm,
        option_lbm,
        option_nbm,
        option_fbm,
        option_ebm,
        option_pb,
    ) in &mut interaction_query
    {
        for (player, mut actions) in actions.iter_mut() {
            if player.id() != 0 {
                continue;
            }

            if Interaction::Clicked != *interaction {
                continue;
            }
            if let Some(_) = option_disabled {
                continue;
            }

            if let Some(_) = option_pbm {
                actions.selected_building = Pulser;
                commands.entity(entity).insert(NewSelectedButton);
            }
            if let Some(_) = option_sbm {
                actions.selected_building = Scatter;
                commands.entity(entity).insert(NewSelectedButton);
            }
            if let Some(_) = option_lbm {
                actions.selected_building = Line;
                commands.entity(entity).insert(NewSelectedButton);
            }

            if let Some(_) = option_nbm {
                actions.selected_ability = Abilities::Nuke;
                commands.entity(entity).insert(NewSelectedButton);
            }
            if let Some(_) = option_fbm {
                actions.selected_ability = Abilities::Fortify;
                commands.entity(entity).insert(NewSelectedButton);
            }
            if let Some(_) = option_ebm {
                actions.selected_ability = Abilities::Expand;
                commands.entity(entity).insert(NewSelectedButton);
            }

            if let Some(_) = option_pb {
                state.set(GamePausedState::Paused);
            }
        }
    }
}

fn handle_selected_button(
    mut commands: Commands,
    mut building_buttons: Query<
        (
            Entity,
            Option<&SelectedButton>,
            Option<&NewSelectedButton>,
            &mut Interaction,
        ),
        (
            With<BuildingButtonsGroupMarker>,
            Without<AbilitiesButtonsGroupMarker>,
        ),
    >,
    mut abilities_buttons: Query<
        (
            Entity,
            Option<&SelectedButton>,
            Option<&NewSelectedButton>,
            &mut Interaction,
        ),
        (
            With<AbilitiesButtonsGroupMarker>,
            Without<BuildingButtonsGroupMarker>,
        ),
    >,
    building_changed: Query<
        &BuildingButtonsGroupMarker,
        (With<BuildingButtonsGroupMarker>, Changed<NewSelectedButton>),
    >,
    abilities_changed: Query<
        &AbilitiesButtonsGroupMarker,
        (
            With<AbilitiesButtonsGroupMarker>,
            Changed<NewSelectedButton>,
        ),
    >,
) {
    if !building_changed.is_empty() {
        //println!("building changed is full");
        for (button, option_old_selection, option_nsb, mut interaction) in
            building_buttons.iter_mut()
        {
            if let Some(_) = option_old_selection {
                commands.entity(button).remove::<SelectedButton>();
                *interaction = Interaction::None;
            }

            if let Some(_) = option_nsb {
                commands.entity(button).insert(SelectedButton);
                commands.entity(button).remove::<NewSelectedButton>();
                *interaction = Interaction::None;
            }
        }
    }

    if !abilities_changed.is_empty() {
        for (button, option_old_selection, option_nsb, mut interaction) in
            abilities_buttons.iter_mut()
        {
            if let Some(_) = option_old_selection {
                commands.entity(button).remove::<SelectedButton>();
                *interaction = Interaction::None;
            }

            if let Some(_) = option_nsb {
                commands.entity(button).insert(SelectedButton);
                commands.entity(button).remove::<NewSelectedButton>();
                *interaction = Interaction::None;
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, button: Query<Entity, With<GameUiThing>>) {
    for button in button.iter() {
        commands.entity(button).despawn_recursive();
    }
}

fn handle_player_cubes_and_stats(
    mut commands: Commands,
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
    player_cubes_parent: Query<(Entity, &PlayerCubesParent)>,
    action_queries: Query<(&PlayerMarker, &Actions), Without<Player>>,
    font_assets: Res<FontAssets>,
    game: Res<GameData>,
    game_build_settings: Res<GameBuildSettings>,
) {
    let Ok((entity, player_cubes_parent)) = player_cubes_parent.get_single() else {
        return;
    };
    commands.entity(entity).despawn_descendants();

    commands.entity(entity).with_children(|mut parent| {
        generate_all_player_cubes(
            &mut parent,
            &player_colors,
            &tile_queries,
            &player_queries,
            &font_assets,
            &game,
            &game_build_settings,
        );
    });
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
    font_assets: &Res<FontAssets>,
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
        if player_query.id() == 0 {
            parent.spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(90.0), Val::Px(5.0)),
                    position_type: PositionType::Relative,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::new(Val::Px(10.0), Val::Px(35.0), Val::Px(10.0), Val::Px(10.0)),
                    //padding: UiRect::right(Val::Px(65.0)),
                    ..default()
                },
                background_color: Color::DARK_GRAY.into(),
                ..default()
            });
        }

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
        if player_query.id() == 0 {
            for (player_query, player_points) in player_queries.iter() {
                if player_query.id() == 0 {
                    parent.spawn(
                        TextBundle::from_section(
                            format!("Building Points: {}", player_points.building_points),
                            TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 40.0,
                                color: player_colors.get_color(0),
                            },
                        )
                        .with_text_alignment(TextAlignment::Center)
                        .with_style(Style {
                            position_type: PositionType::Relative,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::all(Val::Px(10.0)),
                            size: Size::new(Val::Auto, Val::Auto),
                            ..default()
                        }),
                    );
                    parent.spawn(
                        TextBundle::from_section(
                            format!("Ability Points: {}", player_points.ability_points),
                            TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 40.0,
                                color: player_colors.get_color(0),
                            },
                        )
                        .with_text_alignment(TextAlignment::Center)
                        .with_style(Style {
                            position_type: PositionType::Relative,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::all(Val::Px(10.0)),
                            size: Size::new(Val::Auto, Val::Auto),
                            ..default()
                        }),
                    );
                }
            }
        }
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
                flex_direction: FlexDirection::RowReverse,
                margin: UiRect::all(Val::Px(5.0)),
                padding: UiRect::right(Val::Px(35.0)),
                ..default()
            },
            background_color: Color::rgba(0.65, 0.65, 0.1, 0.0).into(),
            ..default()
        })
        .with_children(|parent| {
            let player_tile_count = player_tile_count as f32 / max_tile_count as f32;
            let mut color = Color::WHITE;

            if player_tile_count > 0.0 {
                color = player_colors.get_color(player_id);
            };
            parent.spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(10.0), Val::Px(25.0)),
                    position_type: PositionType::Relative,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                background_color: BackgroundColor::from(color),
                ..default()
            });

            for index in 1..=10 {
                let mut color = Color::WHITE;

                if index as f32 / 10.0 < player_tile_count {
                    color = player_colors.get_color(player_id);
                };

                parent.spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(25.0), Val::Px(25.0)),
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    background_color: BackgroundColor::from(color),
                    ..default()
                });
            }
        });
}

fn game_button<T, B>(
    parent: &mut ChildBuilder,
    font_assets: &Res<FontAssets>,
    menu_type: T,
    button_marker: B,
    button_text: &str,
    button_icon: Handle<Image>,
    building_cost: u32,
    bundle: Option<impl Bundle>,
) -> Entity
where
    T: Bundle,
    B: Bundle,
{
    let mut new_button = parent.spawn_empty();
    new_button
        .insert(ButtonBundle {
            style: Style {
                size: Size::new(Val::Auto, Val::Auto),
                margin: UiRect::all(Val::Px(30.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                position_type: PositionType::Relative,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            background_color: Color::rgba(0.65, 0.65, 0.1, 0.0).into(),
            ..default()
        })
        .insert(menu_type)
        .insert(button_marker)
        .insert(GameButton)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Auto, Val::Px(50.0)),
                        padding: UiRect::new(
                            Val::Px(35.0),
                            Val::Px(10.0),
                            Val::Px(10.0),
                            Val::Px(10.0),
                        ),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    background_color: Color::GRAY.into(),
                    ..default()
                })
                .insert(MainButtonBG)
                .with_children(|parent| {
                    parent
                        .spawn(
                            (TextBundle::from_section(
                                button_text,
                                TextStyle {
                                    font: font_assets.fira_sans.clone(),
                                    font_size: 52.0,
                                    color: Color::BLACK,
                                },
                            )
                            .with_text_alignment(TextAlignment::Center)
                            .with_style(Style {
                                size: Size::new(Val::Auto, Val::Auto),
                                position_type: PositionType::Relative,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(5.0)),
                                align_self: AlignSelf::Center,
                                ..default()
                            })),
                        )
                        .insert(MainButtonText);
                });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(50.0), Val::Px(50.0)),
                        margin: UiRect::all(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        flex_direction: FlexDirection::Row,
                        position: UiRect::new(
                            Val::Px(-35.0),
                            Val::Px(0.0),
                            Val::Px(-35.0),
                            Val::Px(0.0),
                        ),
                        ..default()
                    },
                    background_color: Color::WHITE.into(),
                    ..default()
                })
                .insert(GameButtonIcon)
                .with_children(|parent| {
                    parent.spawn(ImageBundle {
                        style: Style {
                            size: Size::new(Val::Px(50.0), Val::Px(50.0)),
                            margin: UiRect::all(Val::Px(10.0)),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::FlexStart,
                            position_type: PositionType::Relative,
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        image: UiImage {
                            texture: button_icon,
                            flip_x: false,
                            flip_y: false,
                        },
                        ..default()
                    });
                });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(35.0), Val::Px(35.0)),
                        padding: UiRect::all(Val::Px(3.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        flex_direction: FlexDirection::Row,
                        position: UiRect::new(
                            Val::Px(35.0),
                            Val::Px(0.0),
                            Val::Px(-30.0),
                            Val::Px(0.0),
                        ),
                        ..default()
                    },
                    background_color: Color::WHITE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                position_type: PositionType::Relative,
                                ..default()
                            },
                            background_color: Color::BLACK.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(
                                (TextBundle::from_section(
                                    format!("{:?}", building_cost),
                                    TextStyle {
                                        font: font_assets.fira_sans.clone(),
                                        font_size: 18.0,
                                        color: Color::WHITE,
                                    },
                                )
                                .with_text_alignment(TextAlignment::Center)
                                .with_style(Style {
                                    size: Size::new(Val::Auto, Val::Auto),
                                    position_type: PositionType::Relative,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    align_self: AlignSelf::Center,
                                    ..default()
                                })),
                            );
                        });
                });
        });

    if let Some(bundle) = bundle {
        new_button.insert(bundle);
    }

    new_button.id()
}
