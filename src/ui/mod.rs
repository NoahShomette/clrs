mod menu;

use crate::loading::colors_loader::{PalettesAssets, PalettesHandle};
use crate::ui::menu::{handle_menu, setup_menu};
use crate::GameState;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_menu.in_schedule(OnEnter(GameState::Menu)))
            .add_system(handle_menu.in_set(OnUpdate(GameState::Menu)));
    }
}

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
        let mut palettes = cell
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

/*
impl Default for PlayerColors {
    fn default() -> Self {
        Self {
            palette_index: 0,
            current_palette: Palette {
                player_colors: vec![
                    String::from("d3bf77"),
                    String::from("657a85"),
                    String::from("5e9d6a"),
                    String::from("45344a"),
                ],
                noncolorable_tile: "000000".to_string(),
                colorable_tile: "272135".to_string(),
            },
            palettes: vec![
                Palette {
                    player_colors: vec![
                        String::from("d3bf77"),
                        String::from("657a85"),
                        String::from("5e9d6a"),
                        String::from("45344a"),
                    ],
                    noncolorable_tile: "000000".to_string(),
                    colorable_tile: "272135".to_string(),
                },
                Palette {
                    player_colors: vec![
                        String::from("00177c"),
                        String::from("84396c"),
                        String::from("598344"),
                        String::from("d09071"),
                    ],
                    noncolorable_tile: "000000".to_string(),
                    colorable_tile: "272135".to_string(),
                },
                Palette {
                    player_colors: vec![
                        String::from("425e9a"),
                        String::from("39a441"),
                        String::from("de9139"),
                        String::from("e6cb47"),
                    ],
                    noncolorable_tile: "000000".to_string(),
                    colorable_tile: "272135".to_string(),
                },
                Palette {
                    player_colors: vec![
                        String::from("0392cf"),
                        String::from("ee4035"),
                        String::from("7bc043"),
                        String::from("f37736"),
                    ],
                    noncolorable_tile: "000000".to_string(),
                    colorable_tile: "fdf498".to_string(),
                },
                Palette {
                    player_colors: vec![
                        String::from("fff200"),
                        String::from("e500ff"),
                        String::from("00ddff"),
                        String::from("000000"),
                    ],
                    noncolorable_tile: "000000".to_string(),
                    colorable_tile: "ffffff".to_string(),
                },
            ],
        }
    }
}

 */

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
