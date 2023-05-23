use crate::abilities::Abilities;
use crate::actions::Actions;
use crate::buildings;
use crate::buildings::BuildingTypes;
use crate::buildings::BuildingTypes::{Line, Pulser, Scatter};
use crate::game::GameBuildSettings;
use crate::level_loader::{LevelHandle, Levels};
use crate::loading::{FontAssets, TextureAssets};
use crate::GameState;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_ggf::player::PlayerMarker;
use bevy_tweening::lens::TransformScaleLens;
use bevy_tweening::{Animator, EaseFunction, RepeatCount, RepeatStrategy, Tween};
use std::ops::DerefMut;
use std::time::Duration;

use crate::ui::{DisabledButton, GameButton, GameButtonIcon, PlayerColors, SelectedButton};

pub struct GameUiPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_menu.in_schedule(OnEnter(GameState::Playing)))
            .add_system(button_interaction.in_set(OnUpdate(GameState::Playing)))
            .add_system(cleanup_menu.in_schedule(OnExit(GameState::Playing)));

        app.add_system(handle_button_visuals.in_set(OnUpdate(GameState::Playing)));
        app.add_system(handle_selected_button.in_set(OnUpdate(GameState::Playing)));
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
    mut background_color_query: Query<(&mut BackgroundColor, &Children), Without<GameButtonIcon>>,
    mut children_text_color: Query<&mut Text>,
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

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    player_colors: Res<PlayerColors>,
    texture_assets: Res<TextureAssets>,
) {
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
                            let pulser_button = game_button(
                                parent,
                                &font_assets,
                                GameUiThing,
                                (PulserButtonMarker, BuildingButtonsGroupMarker),
                                "Pulser",
                                texture_assets.pulser.clone(),
                            );
                            commands.entity(pulser_button).insert(NewSelectedButton);
                            game_button(
                                parent,
                                &font_assets,
                                GameUiThing,
                                (ScatterButtonMarker, BuildingButtonsGroupMarker),
                                "Scatter",
                                texture_assets.scatter.clone(),
                            );
                            game_button(
                                parent,
                                &font_assets,
                                GameUiThing,
                                (LineButtonMarker, BuildingButtonsGroupMarker),
                                "Line",
                                texture_assets.line.clone(),
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
                            let nuke_entity = game_button(
                                parent,
                                &font_assets,
                                GameUiThing,
                                (NukeButtonMarker, AbilitiesButtonsGroupMarker),
                                "Nuke",
                                texture_assets.nuke.clone(),
                            );
                            commands.entity(nuke_entity).insert(NewSelectedButton);

                            game_button(
                                parent,
                                &font_assets,
                                GameUiThing,
                                (FortifyButtonMarker, AbilitiesButtonsGroupMarker),
                                "Fortify",
                                texture_assets.fortify.clone(),
                            );
                            game_button(
                                parent,
                                &font_assets,
                                GameUiThing,
                                (ExpandButtonMarker, AbilitiesButtonsGroupMarker),
                                "Expand",
                                texture_assets.expand.clone(),
                            );
                        });
                });

            // Node wrapping entire right side ui
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
                                .insert(GameUiThing)
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section(
                                        "PLAY",
                                        TextStyle {
                                            font: font_assets.fira_sans.clone(),
                                            font_size: 40.0,
                                            color: Color::BLACK,
                                        },
                                    ));
                                });
                        });
                });
        });
}

fn button_interaction(
    mut state: ResMut<NextState<GameState>>,
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
        (&BuildingButtonsGroupMarker),
        (With<BuildingButtonsGroupMarker>, Changed<NewSelectedButton>),
    >,
    abilities_changed: Query<
        (&AbilitiesButtonsGroupMarker),
        (
            With<AbilitiesButtonsGroupMarker>,
            Changed<NewSelectedButton>,
        ),
    >,
) {
    if !building_changed.is_empty() {
        println!("building changed is full");
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

fn game_button<T, B>(
    parent: &mut ChildBuilder,
    font_assets: &Res<FontAssets>,
    menu_type: T,
    button_marker: B,
    button_text: &str,
    button_icon: Handle<Image>,
) -> Entity
where
    T: Component,
    B: Bundle,
{
    parent
        .spawn(ButtonBundle {
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
                .with_children(|parent| {
                    parent.spawn(
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
                    );
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
        })
        .id()
}
