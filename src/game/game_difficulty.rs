use std::fmt::Display;

use bevy::reflect::Reflect;

#[derive(Reflect, Clone, Debug, PartialEq)]
pub enum GameDifficulty {
    Easy,
    Medium,
    Hard,
}

impl Display for GameDifficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameDifficulty::Easy => f.write_str("Easy"),
            GameDifficulty::Medium => f.write_str("Medium"),
            GameDifficulty::Hard => f.write_str("Hard"),
        }
    }
}

impl GameDifficulty {
    /// The chance that an AI will act in this tick
    pub fn ai_building_speed(&self) -> f64 {
        match self {
            GameDifficulty::Easy => 0.001,
            GameDifficulty::Medium => 0.01,
            GameDifficulty::Hard => 0.05,
        }
    }

    /// The chance that an AI will act in this tick
    pub fn ai_action_speed(&self) -> f64 {
        match self {
            GameDifficulty::Easy => 0.01,
            GameDifficulty::Medium => 0.1,
            GameDifficulty::Hard => 0.5,
        }
    }

    pub fn increase_difficulty(&self) -> GameDifficulty {
        match self {
            GameDifficulty::Easy => GameDifficulty::Medium,
            GameDifficulty::Medium => GameDifficulty::Hard,
            GameDifficulty::Hard => GameDifficulty::Hard,
        }
    }

    pub fn decrease_difficulty(&self) -> GameDifficulty {
        match self {
            GameDifficulty::Easy => GameDifficulty::Easy,
            GameDifficulty::Medium => GameDifficulty::Easy,
            GameDifficulty::Hard => GameDifficulty::Medium,
        }
    }
}
