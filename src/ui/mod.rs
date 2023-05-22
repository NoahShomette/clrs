mod game;
mod menu;

use crate::game::GameBuildSettings;
use crate::loading::colors_loader::{PalettesAssets, PalettesHandle};
use crate::loading::FontAssets;
use crate::ui::game::GameUiPlugin;
use crate::ui::menu::MenuPlugin;
use crate::GameState;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};
use bevy_tweening::lens::TransformScaleLens;
use bevy_tweening::{Animator, EaseFunction, RepeatCount, RepeatStrategy, Tween};
use std::time::Duration;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MenuPlugin).add_plugin(GameUiPlugin);
        app.add_system(handle_button_visuals.in_set(OnUpdate(GameState::Menu)));
        app.add_system(scale);

        app.add_system(modal_button_interaction);

        /*
        app.add_plugin(ResourceInspectorPlugin::<GameBuildSettings>::default())
            .add_plugin(WorldInspectorPlugin::new());

         */
    }
}

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

/// A marker marking a modal close button. Contains a reference ot the root modal entity
#[derive(Component)]
struct ModalCloseButtonMarker(Entity);

fn modal_panel<T>(
    menu_type: T,
    with_close_button: bool,
    mut commands: &mut Commands,
    font_assets: &Res<FontAssets>,
) -> Entity
where
    T: Component,
{
    //root node for the entire main menu
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: Color::BLACK.with_a(0.3).into(),
            focus_policy: FocusPolicy::Block,
            ..default()
        })
        .insert(menu_type)
        .with_children(|master_parent| {
            let parent_entity = master_parent.parent_entity();
            //root node for the main controls wrapping the entire control section on the left side
            master_parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(60.0), Val::Percent(80.0)),
                        justify_content: JustifyContent::End,
                        align_items: AlignItems::Start,
                        position_type: PositionType::Relative,
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..default()
                })
                .with_children(|parent| {
                    if with_close_button {
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
                    }
                });
        })
        .id()
}