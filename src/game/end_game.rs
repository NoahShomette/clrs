use crate::game::GameBuildSettings;
use crate::mapping::map::MapTileStorage;
use crate::mapping::MapTileIndex;
use crate::objects::{ObjectIndex, TileToObjectIndex};
use crate::GameState;
use bevy::ecs::system::Res;
use bevy::prelude::{
    Commands, DespawnRecursiveExt, Entity, NextState, Query, ResMut, Resource, With,
};
use bevy::reflect::Reflect;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ggf::game_core::Game;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::object::Object;
use bevy_ggf::player::{Player, PlayerMarker};

#[derive(Reflect, Clone, Debug, PartialEq)]
pub enum GameEndConditions {
    Domination,
    Percentage { target_percentage: f32 },
}

#[derive(Default, Resource)]
pub struct GameEnded {
    pub player_won: bool,
    pub winning_id: usize,
}

pub fn cleanup_game(
    tiles: Query<Entity, With<TilePos>>,
    objects: Query<Entity, With<Object>>,
    players: Query<Entity, With<Player>>,
    player_zero: Query<Entity, With<PlayerMarker>>,
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
    commands.remove_resource::<MapTileStorage>();
    commands.init_resource::<GameBuildSettings>();
    commands.insert_resource(MapTileIndex::default());
    commands.insert_resource(ObjectIndex::default());
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
pub fn update_game_end_state(
    tiles: Query<&PlayerMarker, With<Tile>>,
    mut commands: Commands,
    game_settings: Res<GameBuildSettings>,
) {
    let mut player_tiles: HashMap<usize, u32> = HashMap::new();
    for player_marker in tiles.iter() {
        let count = player_tiles.entry(player_marker.id()).or_insert(0);
        let count = *count;
        player_tiles.insert(player_marker.id(), count.saturating_add(1));
    }

    // no matter what conditions we are in if the player is dead we just lose the game
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
    }

    match game_settings.game_end_conditions {
        GameEndConditions::Domination => {
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
        GameEndConditions::Percentage { target_percentage } => {
            let tile_count = game_settings.map_size * game_settings.map_size;
            for (id, count) in player_tiles.iter() {
                if *count as f32 / tile_count as f32 >= target_percentage {
                    let player_won = *id == 0;
                    commands.insert_resource(GameEnded {
                        player_won,
                        winning_id: *id,
                    });
                    return;
                }
            }

            // We also check the tiles to see if the player is the last player. If they are then we end the game early.
            // Because dead players cant gain money theres no point to keeping playing 
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
}
