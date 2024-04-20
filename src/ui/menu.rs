use crate::game::{setup_game_resource, GameBuildSettings};
use crate::level_loader::{LevelHandle, Levels};
use crate::loading::FontAssets;
use crate::GameState;
use bevy::app::AppExit;
use bevy::ecs::system::Insert;
use bevy::prelude::*;

use crate::ui::{modal_panel, BasicButton, DisabledButton, ModalStyle, PlayerColors};

pub struct MenuPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            setup_menu
                .in_schedule(OnEnter(GameState::Menu))
                .after(setup_game_resource),
        )
        .add_systems(
            (
                click_play_button,
                apply_system_buffers,
                update_map_size,
                update_enemies_count,
                update_map_name,
                update_color_swatches,
                update_title_text_colors,
            )
                .chain()
                .in_set(OnUpdate(GameState::Menu)),
        )
        .add_system(cleanup_menu.in_schedule(OnExit(GameState::Menu)));
    }
}

#[derive(Component)]
struct MenuUiThing;

#[derive(Component)]
struct PlayButton;

#[derive(Component)]
struct NextMapButton;

#[derive(Component)]
struct PrevMapButton;

#[derive(Component)]
struct IncreaseMapSizeButton;

#[derive(Component)]
struct DecreaseMapSizeButton;

#[derive(Component)]
struct IncreasePlayerCountButton;

#[derive(Component)]
struct DecreasePlayerCountButton;

#[derive(Component)]
struct NextColorButton;

#[derive(Component)]
struct PrevColorButton;

#[derive(Component)]
struct QuitButton;

#[derive(Component)]
struct SettingsButton;

#[derive(Component)]
struct MapText;

#[derive(Component)]
struct MapSizeText;

#[derive(Component)]
struct PlayerCountText;

#[derive(Component)]
struct ColorSwatch(u8);

#[derive(Component)]
struct UpdateMapSizeButtonColors;

#[derive(Component)]
struct SettingsCloseButton;

#[derive(Component)]
struct UpdateTitleColors;

pub fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    player_colors: Res<PlayerColors>,
    game_build_settings: ResMut<GameBuildSettings>,
    level_handles: Res<LevelHandle>,
    level_assets: Res<Assets<Levels>>,
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
        .insert(MenuUiThing)
        .with_children(|parent| {
            //root node for the main controls wrapping the entire control section on the left side
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
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
                    parent
                        .spawn(
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
                                margin: UiRect::top(Val::Px(75.0)),
                                size: Size::new(Val::Auto, Val::Auto),
                                ..default()
                            }),
                        )
                        .insert(UpdateTitleColors);

                    // node wrapping the actual buttons
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(50.0), Val::Percent(80.0)),
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
                                .insert(MenuUiThing)
                                .insert(PlayButton)
                                .insert(BasicButton)
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
                            let backward_enabled = !game_build_settings.map_type == 0;
                            let forward_enabled = game_build_settings.map_type > 0
                                && game_build_settings.map_type < game_build_settings.max_map - 1;

                            back_and_forth_button(
                                parent,
                                &font_assets,
                                MenuUiThing,
                                PrevMapButton,
                                backward_enabled,
                                NextMapButton,
                                forward_enabled,
                                "MAP",
                            );

                            parent
                                .spawn(
                                    TextBundle::from_section(
                                        level_assets.get(&level_handles.levels).unwrap().levels
                                            [game_build_settings.map_type]
                                            .name
                                            .as_str(),
                                        TextStyle {
                                            font: font_assets.fira_sans.clone(),
                                            font_size: 40.0,
                                            color: Color::GRAY,
                                        },
                                    )
                                    .with_text_alignment(TextAlignment::Center)
                                    .with_style(Style {
                                        position_type: PositionType::Relative,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::all(Val::Px(5.0)),
                                        size: Size::new(Val::Auto, Val::Auto),
                                        ..default()
                                    }),
                                )
                                .insert(MapText);

                            let mut backward = game_build_settings.map_size > 30;
                            let mut forward = game_build_settings.map_size < 100;
                            if game_build_settings.map_type > 1 {
                                backward = false;
                                forward = false;
                            }
                            back_and_forth_button(
                                parent,
                                &font_assets,
                                MenuUiThing,
                                DecreaseMapSizeButton,
                                backward,
                                IncreaseMapSizeButton,
                                forward,
                                "SIZE",
                            );

                            parent
                                .spawn(
                                    TextBundle::from_section(
                                        format!(
                                            "{}x{}",
                                            game_build_settings.map_size,
                                            game_build_settings.map_size
                                        ),
                                        TextStyle {
                                            font: font_assets.fira_sans.clone(),
                                            font_size: 40.0,
                                            color: Color::GRAY,
                                        },
                                    )
                                    .with_text_alignment(TextAlignment::Center)
                                    .with_style(Style {
                                        position_type: PositionType::Relative,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::all(Val::Px(5.0)),
                                        size: Size::new(Val::Auto, Val::Auto),
                                        ..default()
                                    }),
                                )
                                .insert(MapSizeText);

                            let backward = game_build_settings.enemy_count != 1;
                            let forward = game_build_settings.enemy_count != 3;

                            back_and_forth_button(
                                parent,
                                &font_assets,
                                MenuUiThing,
                                DecreasePlayerCountButton,
                                backward,
                                IncreasePlayerCountButton,
                                forward,
                                "ENEMIES",
                            );

                            parent
                                .spawn(
                                    TextBundle::from_section(
                                        format!("{}", game_build_settings.enemy_count,),
                                        TextStyle {
                                            font: font_assets.fira_sans.clone(),
                                            font_size: 40.0,
                                            color: Color::GRAY,
                                        },
                                    )
                                    .with_text_alignment(TextAlignment::Center)
                                    .with_style(Style {
                                        position_type: PositionType::Relative,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::all(Val::Px(5.0)),
                                        size: Size::new(Val::Auto, Val::Auto),
                                        ..default()
                                    }),
                                )
                                .insert(PlayerCountText);

                            let backward = player_colors.palette_index != 0;
                            let forward =
                                player_colors.palette_index != player_colors.palettes.len() - 1;

                            back_and_forth_button(
                                parent,
                                &font_assets,
                                MenuUiThing,
                                PrevColorButton,
                                backward,
                                NextColorButton,
                                forward,
                                "CLRS",
                            );

                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Auto, Val::Auto),
                                        position_type: PositionType::Relative,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    },
                                    background_color: Color::rgba(0.65, 0.65, 0.1, 0.0).into(),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent
                                        .spawn(NodeBundle {
                                            style: Style {
                                                size: Size::new(Val::Px(50.0), Val::Px(50.0)),
                                                position_type: PositionType::Relative,
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                margin: UiRect::all(Val::Px(5.0)),
                                                ..default()
                                            },
                                            background_color: BackgroundColor::from(
                                                player_colors.get_color(0),
                                            ),
                                            ..default()
                                        })
                                        .insert(ColorSwatch(0));
                                    parent
                                        .spawn(NodeBundle {
                                            style: Style {
                                                size: Size::new(Val::Px(50.0), Val::Px(50.0)),
                                                position_type: PositionType::Relative,
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                margin: UiRect::all(Val::Px(5.0)),
                                                ..default()
                                            },
                                            background_color: BackgroundColor::from(
                                                player_colors.get_color(1),
                                            ),
                                            ..default()
                                        })
                                        .insert(ColorSwatch(1));
                                    parent
                                        .spawn(NodeBundle {
                                            style: Style {
                                                size: Size::new(Val::Px(50.0), Val::Px(50.0)),
                                                position_type: PositionType::Relative,
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                margin: UiRect::all(Val::Px(5.0)),
                                                ..default()
                                            },
                                            background_color: BackgroundColor::from(
                                                player_colors.get_color(2),
                                            ),
                                            ..default()
                                        })
                                        .insert(ColorSwatch(2));
                                    parent
                                        .spawn(NodeBundle {
                                            style: Style {
                                                size: Size::new(Val::Px(50.0), Val::Px(50.0)),
                                                position_type: PositionType::Relative,
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                margin: UiRect::all(Val::Px(5.0)),
                                                ..default()
                                            },
                                            background_color: BackgroundColor::from(
                                                player_colors.get_color(3),
                                            ),
                                            ..default()
                                        })
                                        .insert(ColorSwatch(3));
                                });

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

                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Auto, Val::Auto),
                                        position_type: PositionType::Relative,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    },
                                    background_color: Color::rgba(0.65, 0.65, 0.1, 0.0).into(),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    #[cfg(not(target_arch = "wasm32"))]
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
                                        .insert(MenuUiThing)
                                        .insert(QuitButton)
                                        .insert(BasicButton)
                                        .with_children(|parent| {
                                            parent.spawn(TextBundle::from_section(
                                                "QUIT",
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
                                        .insert(MenuUiThing)
                                        .insert(SettingsButton)
                                        .insert(BasicButton)
                                        .with_children(|parent| {
                                            parent.spawn(TextBundle::from_section(
                                                "SETTINGS",
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
        });
}

fn click_play_button(
    mut state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
    mut game_build_settings: ResMut<GameBuildSettings>,
    mut player_colors: ResMut<PlayerColors>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            Option<&DisabledButton>,
            Option<&PlayButton>,
            Option<&NextMapButton>,
            Option<&PrevMapButton>,
            Option<&IncreaseMapSizeButton>,
            Option<&DecreaseMapSizeButton>,
            Option<&IncreasePlayerCountButton>,
            Option<&DecreasePlayerCountButton>,
            Option<&NextColorButton>,
            Option<&PrevColorButton>,
            Option<&QuitButton>,
            Option<&SettingsButton>,
        ),
        (Changed<Interaction>, (With<Button>)),
    >,
    font_assets: Res<FontAssets>,
) {
    for (
        _,
        interaction,
        option_disabled,
        option_pb,
        option_nmb,
        option_pmb,
        option_imsb,
        option_dmsb,
        option_ipcb,
        option_dpcb,
        option_ncb,
        option_pcb,
        option_qb,
        option_sb,
    ) in &mut interaction_query
    {
        if Interaction::Clicked != *interaction {
            continue;
        }

        if let Some(_) = option_disabled {
            continue;
        }

        let modifier = keyboard_input.pressed(KeyCode::LShift);

        if let Some(_) = option_pb {
            state.set(GameState::Playing);
        }

        {
            if let Some(_) = option_nmb {
                game_build_settings.next_map();
                if game_build_settings.map_type > 1 {}
            }
            if let Some(_) = option_pmb {
                game_build_settings.prev_map();
                if game_build_settings.map_type <= 1 {}
            }
        }

        {
            if let Some(_) = option_imsb {
                game_build_settings.increase_map_size(modifier);
            }
            if let Some(_) = option_dmsb {
                game_build_settings.decrease_map_size(modifier);
            }
        }

        {
            if let Some(_) = option_ipcb {
                game_build_settings.increase_enemy_count();
            }
            if let Some(_) = option_dpcb {
                game_build_settings.decrease_enemy_count();
            }
        }

        {
            if let Some(_) = option_ncb {
                player_colors.next_palette();
            }
            if let Some(_) = option_pcb {
                player_colors.prev_palette();
            }
        }

        if let Some(_) = option_sb {
            modal_panel(
                MenuUiThing,
                ModalStyle {
                    with_close_button: true,
                    close_button_bundle: None::<SettingsCloseButton>,
                    modal_size: None,
                },
                &mut commands,
                &font_assets,
            );
            //TODO: Add settings after we make the pop up template
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(_) = option_qb {
            exit.send(AppExit);
        }
    }
}

fn update_map_name(
    mut colors: Query<(&MapText, &mut Text)>,
    mut buttons: Query<(
        Entity,
        Option<&DisabledButton>,
        Option<&PrevMapButton>,
        Option<&NextMapButton>,
        &mut BackgroundColor,
    )>,
    mut commands: Commands,
    game_build_settings: Res<GameBuildSettings>,
    level_handles: Res<LevelHandle>,
    level_assets: Res<Assets<Levels>>,
) {
    for (_, mut text) in colors.iter_mut() {
        text.sections[0].value = level_assets.get(&level_handles.levels).unwrap().levels
            [game_build_settings.map_type]
            .name
            .clone();
    }
    for (entity, option_disabled_button, option_1, option_2, mut background_color) in
        buttons.iter_mut()
    {
        if game_build_settings.map_type == 0 {
            if let Some(_) = option_1 {
                if let None = option_disabled_button {
                    background_color.0 = Color::DARK_GRAY;
                    commands.entity(entity).insert(DisabledButton);
                }
            }
            if let Some(_) = option_2 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
        }
        if game_build_settings.map_type == game_build_settings.max_map - 1 {
            if let Some(_) = option_1 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
            if let Some(_) = option_2 {
                if let None = option_disabled_button {
                    background_color.0 = Color::DARK_GRAY;
                    commands.entity(entity).insert(DisabledButton);
                }
            }
        }

        if game_build_settings.map_type > 0
            && game_build_settings.map_type < game_build_settings.max_map - 1
        {
            if let Some(_) = option_1 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
            if let Some(_) = option_2 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
        }
    }
}

fn update_map_size(
    mut colors: Query<(&MapSizeText, &mut Text)>,
    mut buttons: Query<(
        Entity,
        Option<&DisabledButton>,
        Option<&DecreaseMapSizeButton>,
        Option<&IncreaseMapSizeButton>,
        &mut BackgroundColor,
    )>,
    game_build_settings: Res<GameBuildSettings>,
    mut commands: Commands,
) {
    for (_, mut text) in colors.iter_mut() {
        text.sections[0].value = format!(
            "{}x{}",
            game_build_settings.map_size, game_build_settings.map_size
        );
    }
    for (entity, option_disabled_button, option_1, option_2, mut background_color) in
        buttons.iter_mut()
    {
        let mut enable_button: Option<bool> = None;

        if let Some(_) = option_disabled_button {
            if let Some(_) = option_1 {
                if game_build_settings.map_size > 30 {
                    enable_button = Some(true);
                }
            }
            if let Some(_) = option_2 {
                if game_build_settings.map_size < 100 {
                    enable_button = Some(true);
                }
            }
        } else {
            if let Some(_) = option_1 {
                if game_build_settings.map_size > 30 {
                } else {
                    enable_button = Some(false);
                }
            }
            if let Some(_) = option_2 {
                if game_build_settings.map_size < 100 {
                } else {
                    enable_button = Some(false);
                }
            }
        }

        if game_build_settings.map_type > 1 {
            if let Some(_) = option_1 {
                enable_button = Some(false);
            }
            if let Some(_) = option_2 {
                enable_button = Some(false);
            }
        }
        if enable_button.is_some() {
            if enable_button.unwrap() {
                background_color.0 = Color::GRAY;
                commands.entity(entity).remove::<DisabledButton>();
            } else {
                background_color.0 = Color::DARK_GRAY;
                commands.entity(entity).insert(DisabledButton);
            }
        }
    }
}

fn update_enemies_count(
    mut colors: Query<(&PlayerCountText, &mut Text)>,
    mut buttons: Query<(
        Entity,
        Option<&DisabledButton>,
        Option<&DecreasePlayerCountButton>,
        Option<&IncreasePlayerCountButton>,
        &mut BackgroundColor,
    )>,
    game_build_settings: Res<GameBuildSettings>,
    mut commands: Commands,
) {
    for (_, mut text) in colors.iter_mut() {
        text.sections[0].value = format!("{}", game_build_settings.enemy_count);
    }

    for (entity, option_disabled_button, option_1, option_2, mut background_color) in
        buttons.iter_mut()
    {
        if game_build_settings.enemy_count == 1 {
            if let Some(_) = option_1 {
                if let None = option_disabled_button {
                    background_color.0 = Color::DARK_GRAY;
                    commands.entity(entity).insert(DisabledButton);
                }
            }
            if let Some(_) = option_2 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
        }
        if game_build_settings.enemy_count == 3 {
            if let Some(_) = option_1 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
            if let Some(_) = option_2 {
                if let None = option_disabled_button {
                    background_color.0 = Color::DARK_GRAY;
                    commands.entity(entity).insert(DisabledButton);
                }
            }
        }

        if game_build_settings.enemy_count > 1 && game_build_settings.enemy_count < 3 {
            if let Some(_) = option_1 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
            if let Some(_) = option_2 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
        }
    }
}

fn update_color_swatches(
    mut colors: Query<(&ColorSwatch, &mut BackgroundColor)>,
    player_colors: Res<PlayerColors>,
    mut buttons: Query<
        (
            Entity,
            Option<&DisabledButton>,
            Option<&PrevColorButton>,
            Option<&NextColorButton>,
            &mut BackgroundColor,
        ),
        Without<ColorSwatch>,
    >,
    mut commands: Commands,
) {
    for (color_swatch, mut background_color) in colors.iter_mut() {
        background_color.0 = player_colors.get_color(color_swatch.0 as usize);
    }

    for (entity, option_disabled_button, option_1, option_2, mut background_color) in
        buttons.iter_mut()
    {
        if player_colors.palette_index == 0 {
            if let Some(_) = option_1 {
                if let None = option_disabled_button {
                    background_color.0 = Color::DARK_GRAY;
                    commands.entity(entity).insert(DisabledButton);
                }
            }
            if let Some(_) = option_2 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
        }
        if player_colors.palette_index == player_colors.palettes.len() - 1 {
            if let Some(_) = option_1 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
            if let Some(_) = option_2 {
                if let None = option_disabled_button {
                    background_color.0 = Color::DARK_GRAY;
                    commands.entity(entity).insert(DisabledButton);
                }
            }
        }

        if player_colors.palette_index > 0
            && player_colors.palette_index < player_colors.palettes.len() - 1
        {
            if let Some(_) = option_1 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
            if let Some(_) = option_2 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
        }
    }
}

fn update_title_text_colors(
    player_colors: Option<Res<PlayerColors>>,
    mut interaction_query: Query<(Entity, &mut Text), (With<UpdateTitleColors>,)>,
) {
    if let Some(player_colors) = player_colors {
        for (entity, mut text) in &mut interaction_query {
            for (i, mut section) in text.sections.iter_mut().enumerate() {
                if section.style.color != player_colors.get_color(i) {
                    section.style.color = player_colors.get_color(i);
                }
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, button: Query<Entity, With<MenuUiThing>>) {
    for button in button.iter() {
        commands.entity(button).despawn_recursive();
    }
}

fn back_and_forth_button<T, B, F>(
    parent: &mut ChildBuilder,
    font_assets: &Res<FontAssets>,
    menu_type: T,
    back_marker: B,
    back_button_enabled: bool,
    forward_marker: F,
    forward_button_enabled: bool,
    button_text: &str,
) -> Entity
where
    T: Component,
    B: Component,
    F: Component,
{
    parent
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Auto, Val::Auto),
                margin: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                position_type: PositionType::Relative,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            background_color: Color::rgba(0.65, 0.65, 0.1, 0.0).into(),
            ..default()
        })
        .with_children(|parent| {
            let button_color = if back_button_enabled {
                BackgroundColor::from(Color::GRAY)
            } else {
                BackgroundColor::from(Color::DARK_GRAY)
            };
            let mut back_button = parent.spawn(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(50.0), Val::Px(50.0)),
                    margin: UiRect::all(Val::Px(5.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: button_color,
                ..Default::default()
            });
            back_button
                .insert(menu_type)
                .insert(back_marker)
                .insert(BasicButton)
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "<",
                        TextStyle {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            color: Color::BLACK,
                        },
                    ));
                });

            if !back_button_enabled {
                back_button.insert(DisabledButton);
            }

            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Auto, Val::Px(50.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    background_color: Color::DARK_GRAY.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(
                        (TextBundle::from_section(
                            button_text,
                            TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 40.0,
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

            let button_color = if forward_button_enabled {
                BackgroundColor::from(Color::GRAY)
            } else {
                BackgroundColor::from(Color::DARK_GRAY)
            };
            let mut forward_button = parent.spawn(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(50.0), Val::Px(50.0)),
                    margin: UiRect::all(Val::Px(5.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: button_color,
                ..Default::default()
            });
            forward_button
                .insert(forward_marker)
                .insert(BasicButton)
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        ">",
                        TextStyle {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            color: Color::BLACK,
                        },
                    ));
                });

            if !forward_button_enabled {
                forward_button.insert(DisabledButton);
            }
        })
        .id()
}
