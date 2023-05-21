use crate::game::GameBuildSettings;
use crate::GameState;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::{
    Camera, Camera2d, ClearColor, Color, Commands, DespawnRecursiveExt, Entity, Mut, NextState,
    OrthographicProjection, Query, Res, ResMut, Resource, With,
};
use bevy::utils::HashMap;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ggf::game_core::Game;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::object::{Object, ObjectGridPosition};
use bevy_ggf::player::{Player, PlayerMarker};
use bevy_vector_shapes::prelude::Canvas;
use image::imageops::tile;

#[derive(Default, Resource)]
pub struct GameEnded {
    pub player_won: bool,
    pub winning_id: usize,
}

pub fn cleanup_game(
    mut tiles: Query<Entity, With<TilePos>>,
    mut objects: Query<Entity, With<Object>>,
    mut players: Query<Entity, With<Player>>,
    mut player_zero: Query<Entity, With<PlayerMarker>>,
    mut commands: Commands,
) {
    for tile in tiles.iter() {
        commands.entity(tile).despawn_recursive();
    }
    for object in objects.iter() {
        commands.entity(object).despawn_recursive();
    }
    for player in players.iter() {
        commands.entity(player).despawn_recursive();
    }
    for player_zero in player_zero.iter() {
        commands.entity(player_zero).despawn_recursive();
    }

    commands.remove_resource::<Game>();
    commands.init_resource::<GameBuildSettings>();
}

pub fn check_game_ended(
    mut game: ResMut<Game>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if game.game_world.contains_resource::<GameEnded>() {
        let game_ended = game.game_world.remove_resource::<GameEnded>().unwrap();
        commands.insert_resource(game_ended);
        next_state.set(GameState::Ended);
    }
}

// This system runs in the game world to see if someone won
pub fn update_game_end_state(mut tiles: Query<&PlayerMarker, With<Tile>>, mut commands: Commands) {
    let mut player_tiles: HashMap<usize, u32> = HashMap::new();
    for player_marker in tiles.iter() {
        let count = player_tiles.entry(player_marker.id()).or_insert(0);
        let count = *count;
        player_tiles.insert(player_marker.id(), count.saturating_add(1));
    }

    if !player_tiles.contains_key(&0) {
        // ai has won
        let mut highest: (usize, u32) = (0, 0);
        for (id, count) in player_tiles.iter() {
            if count > &highest.1 {
                highest.0 = *id;
                highest.1 = *count;
            }
        }
        commands.insert_resource(GameEnded {
            player_won: false,
            winning_id: highest.0,
        });
    } else {
        player_tiles.remove(&0);
        if player_tiles.is_empty() {
            // player has won
            commands.insert_resource(GameEnded {
                player_won: true,
                winning_id: 0,
            });
        } else {
            // nothing no one has won
        }
    }
}
