mod draw;

use bevy::app::App;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::{TilemapGridSize, TilemapSize, TilemapType, TilePos};
use crate::actions::paused_controls;
use crate::draw::draw::{draw_tiles, draw_objects, TILE_SIZE, TILE_GAP};
use crate::GameState;

pub struct DrawPlugin;

impl Plugin for DrawPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(
            (draw_tiles, draw_objects).in_set(OnUpdate(GameState::Playing))
        );
        app.add_systems(
            (draw_tiles, draw_objects).before(paused_controls)
                .in_set(OnUpdate(GameState::Paused))
        );
        app.add_systems(
            (draw_tiles, draw_objects).in_set(OnUpdate(GameState::Ended))
        );

       // app.add_system(draw_game_over.in_set(OnUpdate(GameState::Ended)));
    }
}

#[derive(Component)]
pub struct DrawTile;

#[derive(Component)]
pub struct DrawObject;

pub fn world_pos_to_tile_pos(world_pos: &Vec2, map_size: &TilemapSize) -> Option<TilePos> {
    let transformed_pos: Vec2 = {
        Vec2 {
            x: world_pos.x + ((map_size.x as f32 * (TILE_SIZE + TILE_GAP)) / 2.0),
            y: world_pos.y + ((map_size.y as f32 * (TILE_SIZE + TILE_GAP)) / 2.0),
        }
    };

    TilePos::from_world_pos(
        &transformed_pos,
        map_size,
        &TilemapGridSize {
            x: TILE_SIZE + TILE_GAP,
            y: TILE_SIZE + TILE_GAP,
        },
        &TilemapType::Square,
    )
}

pub fn tile_pos_to_centered_map_world_pos(tile_pos: &TilePos, map_size: &TilemapSize) -> Vec2 {
    let tile_world_pos = tile_pos
        .center_in_world(
            &TilemapGridSize {
                x: TILE_SIZE + TILE_GAP,
                y: TILE_SIZE + TILE_GAP,
            },
            &TilemapType::Square,
        )
        .extend(0.0);

    let transformed_pos: Vec2 = {
        Vec2 {
            x: tile_world_pos.x - ((map_size.x as f32 * (TILE_SIZE + TILE_GAP)) / 2.0),
            y: tile_world_pos.y - ((map_size.y as f32 * (TILE_SIZE + TILE_GAP)) / 2.0),
        }
    };
    transformed_pos
}
