mod game;
mod menu;

use crate::loading::colors_loader::{PalettesAssets, PalettesHandle};
use crate::ui::game::GameUiPlugin;
use crate::ui::menu::MenuPlugin;
use crate::GameState;
use bevy::prelude::*;
use bevy_tweening::lens::TransformScaleLens;
use bevy_tweening::{Animator, EaseFunction, RepeatCount, RepeatStrategy, Tween};
use std::time::Duration;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MenuPlugin).add_plugin(GameUiPlugin);
        app.add_system(handle_button_visuals.in_set(OnUpdate(GameState::Menu)));
    }
}

#[derive(Component)]
pub struct DisabledButton;

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

fn handle_button_visuals(
    player_colors: Option<Res<PlayerColors>>,
    mut interaction_query: Query<
        (Entity, &Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<Button>, Without<DisabledButton>),
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
