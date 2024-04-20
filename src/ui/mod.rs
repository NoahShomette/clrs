mod components;
mod end_game;
mod game;
mod menu;
mod pause;
mod widgets;

use crate::loading::colors_loader::{PalettesAssets, PalettesHandle};
use crate::loading::FontAssets;
use crate::ui::end_game::EndGameUiPlugin;
use crate::ui::game::GameUiPlugin;
use crate::ui::menu::MenuPlugin;
use crate::ui::pause::PauseUiPlugin;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::window::PrimaryWindow;
use bevy_tweening::lens::TransformScaleLens;
use bevy_tweening::{Animator, EaseFunction, RepeatCount, RepeatStrategy, Tween};
use std::time::Duration;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MenuPlugin)
            .add_plugin(GameUiPlugin)
            .add_plugin(PauseUiPlugin)
            .add_plugin(EndGameUiPlugin);

        app.add_systems((
            handle_button_visuals,
            scale,
            modal_button_interaction,
            handle_background_colors,
            handle_text_colors,
        ));

        /*
        app.add_plugin(ResourceInspectorPlugin::<GameBuildSettings>::default())
            .add_plugin(WorldInspectorPlugin::new());
         */
    }
}

#[derive(Component)]
pub struct UpdateBackgroundWithCurrentPlayerColor;

#[derive(Component)]
pub struct UpdateTextColorWithCurrentPlayerColor;

#[derive(Component)]
pub struct DisabledButton;

#[derive(Component)]
pub struct SelectedButton;

#[derive(Component)]
pub struct BasicButton;

#[derive(Component)]
pub struct GameButton;

#[derive(Component)]
pub struct GameButtonIcon;

#[derive(Default)]
pub struct MenuNavigation(pub u32);

#[derive(Clone, Resource)]
pub struct PlayerColors {
    pub palette_index: usize,
    pub current_palette: Palette,
    pub palettes: Vec<Palette>,
}

impl FromWorld for PlayerColors {
    fn from_world(world: &mut World) -> Self {
        let cell = world.cell();
        let palettes = cell
            .get_resource_mut::<Assets<PalettesAssets>>()
            .expect("Failed to get Assets<PalettesAssets>");
        let palettes_handle = cell
            .get_resource::<PalettesHandle>()
            .expect("Failed to get PalettesHandle");

        let palettes = palettes.get(&palettes_handle.palettes).unwrap();

        let mut player_colors = PlayerColors {
            palette_index: 0,
            current_palette: palettes.palettes[0].clone(),
            palettes: vec![],
        };

        for palette in palettes.palettes.iter() {
            player_colors.palettes.push(palette.clone());
        }
        player_colors
    }
}

impl PlayerColors {
    pub fn get_color(&self, player_id: usize) -> Color {
        return Color::hex(self.current_palette.player_colors[player_id].clone()).unwrap();
    }
    pub fn next_palette(&mut self) {
        if self.palette_index.saturating_add(1) < self.palettes.len() {
            self.palette_index = self.palette_index.saturating_add(1);
            self.current_palette = self.palettes[self.palette_index].clone();
        }
    }
    pub fn prev_palette(&mut self) {
        self.palette_index = self.palette_index.saturating_sub(1);
        self.current_palette = self.palettes[self.palette_index].clone();
    }
    pub fn get_noncolorable(&self) -> Color {
        return Color::hex(self.current_palette.noncolorable_tile.clone()).unwrap();
    }
    pub fn get_colorable(&self) -> Color {
        return Color::hex(self.current_palette.colorable_tile.clone()).unwrap();
    }
}

#[derive(Clone, serde::Deserialize)]
pub struct Palette {
    pub player_colors: Vec<String>,
    pub noncolorable_tile: String,
    pub colorable_tile: String,
}

pub fn scale(
    mut cached_size: Local<Vec2>,
    mut ui_scale: ResMut<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Some(primary) = windows.iter().next() else {
        return;
    };
    let ww = primary.width();
    let wh = primary.height();
    if cached_size.x == ww && cached_size.y == wh {
        return;
    }
    cached_size.x = ww;
    cached_size.y = wh;

    let scale_h = ww / 1920.0;
    let scale_w = wh / 1080.0;
    ui_scale.scale = scale_h.min(scale_w) as f64;
}

fn handle_background_colors(
    player_colors: Option<Res<PlayerColors>>,
    mut interaction_query: Query<
        (Entity, &mut BackgroundColor),
        (With<UpdateBackgroundWithCurrentPlayerColor>,),
    >,
) {
    if let Some(player_colors) = player_colors {
        for (entity, mut color) in &mut interaction_query {
            if color.0 != player_colors.get_color(0) {
                color.0 = player_colors.get_color(0);
            }
        }
    }
}

fn handle_text_colors(
    player_colors: Option<Res<PlayerColors>>,
    mut interaction_query: Query<
        (Entity, &mut Text),
        (With<UpdateTextColorWithCurrentPlayerColor>,),
    >,
) {
    if let Some(player_colors) = player_colors {
        for (entity, mut text) in &mut interaction_query {
            for mut section in text.sections.iter_mut() {
                if section.style.color != player_colors.get_color(0) {
                    section.style.color = player_colors.get_color(0);
                }
            }
        }
    }
}

fn handle_button_visuals(
    player_colors: Option<Res<PlayerColors>>,
    mut interaction_query: Query<
        (Entity, &Interaction, &mut BackgroundColor, &Children),
        (
            Changed<Interaction>,
            With<Button>,
            Without<DisabledButton>,
            With<BasicButton>,
        ),
    >,
    mut children_text_color: Query<&mut Text>,
    mut commands: Commands,
) {
    if let Some(player_colors) = player_colors {
        for (entity, interaction, mut color, children) in &mut interaction_query {
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

                    *color = BackgroundColor::from(player_colors.get_color(0));
                    for &child in children.iter() {
                        if let Ok(mut text) = children_text_color.get_mut(child) {
                            text.sections[0].style.color = Color::BLACK;
                        }
                    }
                }
                Interaction::None => {
                    *color = BackgroundColor::from(Color::GRAY);
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

fn modal_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &ModalCloseButtonMarker),
        (Changed<Interaction>, (With<Button>)),
    >,
    mut commands: Commands,
) {
    for (interaction, modal_close_button) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                commands.entity(modal_close_button.0).despawn_recursive();
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

pub struct ModalStyle<B: Bundle> {
    with_close_button: bool,
    close_button_bundle: Option<B>,
    modal_size: Option<Size>,
}

/// A marker marking a modal close button. Contains a reference ot the root modal entity
#[derive(Component)]
struct ModalCloseButtonMarker(Entity);

fn modal_panel<T>(
    menu_type: T,
    modal_style: ModalStyle<impl Bundle>,
    mut commands: &mut Commands,
    font_assets: &Res<FontAssets>,
) -> Entity
where
    T: Component,
{
    //we assign it to a basic entity and then reassign it later
    let mut inside_entity = Entity::from_raw(0);
    //root node for the entire modal

    let modal_size = match modal_style.modal_size {
        None => Size::new(Val::Auto, Val::Auto),
        Some(size) => size,
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: Color::BLACK.with_a(0.5).into(),
            focus_policy: FocusPolicy::Block,
            ..default()
        })
        .insert(menu_type)
        .with_children(|master_parent| {
            let parent_entity = master_parent.parent_entity();

            master_parent
                .spawn(NodeBundle {
                    style: Style {
                        size: modal_size,
                        padding: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Start,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Relative,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: Color::rgba(1.0, 1.0, 1.0, 1.0).into(),
                    ..default()
                })
                .with_children(|parent| {
                    //root node for the inside panel
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                padding: UiRect::all(Val::Px(10.0)),
                                justify_content: JustifyContent::Start,
                                align_items: AlignItems::Center,
                                position_type: PositionType::Relative,
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            background_color: Color::rgba(0.0, 0.0, 0.0, 1.0).into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            if modal_style.with_close_button {
                                // Top option close button
                                parent
                                    .spawn(NodeBundle {
                                        style: Style {
                                            size: Size::new(
                                                Val::Percent(100.0),
                                                Val::Percent(10.0),
                                            ),
                                            justify_content: JustifyContent::End,
                                            align_items: AlignItems::Start,
                                            position_type: PositionType::Relative,
                                            flex_direction: FlexDirection::Row,
                                            ..default()
                                        },
                                        background_color: Color::rgba(0.0, 0.0, 0.0, 0.0).into(),
                                        ..default()
                                    })
                                    .with_children(|parent| {
                                        let mut button_entity = parent.spawn_empty();
                                        button_entity
                                            .insert(ButtonBundle {
                                                style: Style {
                                                    size: Size::new(Val::Auto, Val::Px(50.0)),
                                                    margin: UiRect::new(
                                                        Val::Px(20.0),
                                                        Val::Px(20.0),
                                                        Val::Px(20.0),
                                                        Val::Px(20.0),
                                                    ),
                                                    padding: UiRect::all(Val::Px(10.0)),
                                                    justify_content: JustifyContent::Center,
                                                    align_items: AlignItems::Center,
                                                    ..Default::default()
                                                },
                                                background_color: BackgroundColor::from(
                                                    Color::GRAY,
                                                ),
                                                ..Default::default()
                                            })
                                            .insert(ModalCloseButtonMarker(parent_entity))
                                            .insert(BasicButton)
                                            .with_children(|parent| {
                                                parent.spawn(TextBundle::from_section(
                                                    "CLOSE",
                                                    TextStyle {
                                                        font: font_assets.fira_sans.clone(),
                                                        font_size: 40.0,
                                                        color: Color::BLACK,
                                                    },
                                                ));
                                            });
                                        if let Some(bundle) = modal_style.close_button_bundle {
                                            button_entity.insert(bundle);
                                        }
                                    });
                            }

                            inside_entity = parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                        margin: UiRect::all(Val::Px(10.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        position_type: PositionType::Relative,
                                        flex_direction: FlexDirection::Column,
                                        ..default()
                                    },
                                    background_color: Color::rgba(0.0, 0.0, 0.0, 0.0).into(),
                                    ..default()
                                })
                                .id();
                        });
                });
        });

    inside_entity
}
