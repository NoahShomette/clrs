use bevy::prelude::{Color, Component, FromReflect};
use bevy::reflect::Reflect;


pub enum ColorResult{
    
}

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Reflect, FromReflect)]
pub enum TileColorStrength {
    #[default]
    Neutral,
    One,
    Two,
    Three,
    Four,
    Five,
}


#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct TileColor {
    pub tile_color_strength: TileColorStrength,
}

impl TileColor {
    pub fn damage(&mut self){
        match self.tile_color_strength {
            TileColorStrength::Neutral => {
            }
            TileColorStrength::One => {
                self.tile_color_strength = TileColorStrength::Neutral;
            }
            TileColorStrength::Two => {
                self.tile_color_strength = TileColorStrength::One;
            }
            TileColorStrength::Three => {
                self.tile_color_strength = TileColorStrength::Two;
            }
            TileColorStrength::Four => {
                self.tile_color_strength = TileColorStrength::Three;
            }
            TileColorStrength::Five => {
                self.tile_color_strength = TileColorStrength::Four;
            }
        }
    }

    pub fn strengthen(&mut self){
        match self.tile_color_strength {
            TileColorStrength::Neutral => {
                self.tile_color_strength = TileColorStrength::One;
            }
            TileColorStrength::One => {
                self.tile_color_strength = TileColorStrength::Two;
            }
            TileColorStrength::Two => {
                self.tile_color_strength = TileColorStrength::Three;
            }
            TileColorStrength::Three => {
                self.tile_color_strength = TileColorStrength::Four;
            }
            TileColorStrength::Four => {
                self.tile_color_strength = TileColorStrength::Five;
            }
            TileColorStrength::Five => {
            }
        }
    }
}

pub enum PlayerColors {
    Blue,
    Red,
    Green,
    Purple,
}

impl PlayerColors {
    pub fn get_color(player_id: usize) -> Color {
        return match player_id {
            0 => Color::BLUE,
            1 => Color::RED,
            2 => Color::GREEN,
            _ => Color::INDIGO,
        };
    }
    pub fn get_colors_from(&mut self) -> Color {
        return match self {
            PlayerColors::Blue => Color::BLUE,
            PlayerColors::Red => Color::RED,
            PlayerColors::Green => Color::GREEN,
            PlayerColors::Purple => Color::INDIGO,
        };
    }
}
