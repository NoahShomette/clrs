use std::hash::Hash;

use crate::buildings::{Activate, Building};
use crate::color_system::{convert_tile, ColorConflictEvent, TileColor, TileColorStrength};
use crate::objects::ObjectCachedMap;
use bevy::ecs::event::EventWriter;
use bevy::ecs::system::Commands;
use bevy::prelude::{Component, Entity, FromReflect, Query, Reflect, With, Without};
use bevy_ecs_tilemap::tiles::TileStorage;
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use rand::Rng;
use serde::{Deserialize, Serialize};

use bevy_ecs_tilemap::prelude::{TilePos, TilemapSize};
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::ObjectId;
use bevy_ggf::player::PlayerMarker;

use super::building_pathfinding::PathfindStrengthExt;
use super::Simulate;

#[derive(
    Default,
    Clone,
    Eq,
    Hash,
    Debug,
    PartialEq,
    Component,
    Reflect,
    FromReflect,
    Serialize,
    Deserialize,
)]
pub struct Pulser {
    pub strength: u32,
    pub max_pulse_tiles: u32,
}

impl PathfindStrengthExt for Pulser {
    fn pathfinding_strength(&self) -> u32 {
        self.strength
    }
}

// two parts - we pulse outwards, checking the outside neighbors of each tile. If the outside neighbors
// are not the same player then we damage their color by one. Otherwise at that point we stop.
pub fn simulate_pulsers_from_cache(
    mut tile_storage_query: Query<(Entity, &MapId, &TileStorage, &TilemapSize)>,
    pulsers: Query<
        (
            Entity,
            &ObjectId,
            &PlayerMarker,
            &Building<Pulser>,
            &ObjectCachedMap,
        ),
        (Without<MapId>, With<Activate>, With<Simulate>),
    >,
    mut tiles: Query<
        (
            Entity,
            &TileTerrainInfo,
            Option<(&mut PlayerMarker, &mut TileColor)>,
        ),
        (With<Tile>, Without<Building<Pulser>>, Without<MapId>),
    >,
    mut event_writer: EventWriter<ColorConflictEvent>,
    mut commands: Commands,
) {
    let Some((_, _, tile_storage, _)) = tile_storage_query
        .iter_mut()
        .find(|(_, id, _, _)| id == &&MapId { id: 1 })
    else {
        return;
    };
    let mut rng = rand::thread_rng();

    for (entity, id, player_marker, pulser, cache) in pulsers.iter() {
        commands.entity(entity).remove::<Activate>();

        let mut tiles_changed: u32 = 0;
        let mut target_tiles = vec![];

        for (index, tile) in cache.cache.iter().enumerate() {
            let Some(tile_entity) = tile_storage.get(&Into::<TilePos>::into(*tile)) else {
                continue;
            };

            if let Ok((_, _, options)) = tiles.get_mut(tile_entity) {
                if let Some((tile_player_marker, tile_color)) = options.as_ref() {
                    if player_marker.id() == tile_player_marker.id() {
                        if let TileColorStrength::Five = tile_color.tile_color_strength {
                        } else {
                            target_tiles.push((index, tile));
                        }
                    } else {
                        target_tiles.push((index, tile));
                    }
                } else {
                    target_tiles.push((index, tile));
                }

                if target_tiles.len() >= (pulser.building_type.max_pulse_tiles + 3) as usize {
                    break;
                }
            }
        }

        while target_tiles.len() > pulser.building_type.max_pulse_tiles as usize {
            let removal_index: usize = rng.gen_range(1..target_tiles.len());
            target_tiles.remove(removal_index);
        }

        for (_, tile) in target_tiles.iter() {
            let Some(tile_entity) = tile_storage.get(&Into::<TilePos>::into(**tile)) else {
                continue;
            };

            if let Ok((_, tile_terrain_info, options)) = tiles.get_mut(tile_entity) {
                if convert_tile(
                    id,
                    &player_marker.id(),
                    Into::<TilePos>::into(**tile),
                    tile_terrain_info,
                    &options,
                    &mut event_writer,
                ) {
                    tiles_changed += 1;
                }
            }
        }

        if tiles_changed == 0 {
            commands.entity(entity).remove::<Simulate>();
        }
    }
}
