mod draw;

use bevy::app::App;
use bevy::prelude::*;
use crate::actions::paused_controls;
use crate::draw::draw::{draw_game, draw_game_over};
use crate::GameState;

pub struct DrawPlugin;

impl Plugin for DrawPlugin{
    fn build(&self, app: &mut App) {
        app.add_system(draw_game.in_set(OnUpdate(GameState::Playing)));
        app.add_system(
            draw_game
                .before(paused_controls)
                .in_set(OnUpdate(GameState::Paused)),
        );
        app.add_system(draw_game.in_set(OnUpdate(GameState::Ended)));

        app.add_system(draw_game_over.in_set(OnUpdate(GameState::Ended)));
    }
}

#[derive(Component)]
pub struct DrawTile;

#[derive(Component)]
pub struct DrawObject;