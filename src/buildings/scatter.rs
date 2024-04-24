use crate::buildings::{check_is_colorable, Activate, Building};
use crate::color_system::{convert_tile, ColorConflictEvent, TileColor, TileColorStrength};
use crate::objects::ObjectCachedMap;
use bevy::prelude::{
    Commands, Component, Entity, EventWriter, FromReflect, Query, Reflect, With, Without,
};
use bevy_ecs_tilemap::prelude::{TileStorage, TilemapSize};
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::ObjectId;
use bevy_ggf::player::PlayerMarker;
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::building_pathfinding::SimpleBuildingPathfindMapExt;
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
pub struct Scatter {
    pub scatter_range: u32,
    pub scatter_amount: u32,
}

impl SimpleBuildingPathfindMapExt for Scatter {
    fn building_strength(&self) -> u32 {
        self.scatter_range
    }
}

// two parts - we pulse outwards, checking the outside neighbors of each tile. If the outside neighbors
// are not the same player then we damage their color by one. Otherwise at that point we stop.
pub fn simulate_scatter_from_cache(
    mut tile_storage_query: Query<(Entity, &MapId, &TileStorage, &TilemapSize)>,
    pulsers: Query<
        (
            Entity,
            &ObjectId,
            &PlayerMarker,
            &Building<Scatter>,
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
        (With<Tile>, Without<Building<Scatter>>, Without<MapId>),
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

    for (entity, id, player_marker, scatter, cache) in pulsers.iter() {
        commands.entity(entity).remove::<Activate>();

        let mut tiles_changed: u32 = 0;

        let mut rng = rand::thread_rng();

        for _ in 0..=scatter.building_type.scatter_amount {
            let y: usize = rng.gen_range(0..cache.cache.len());

            let Some(tile_entity) = tile_storage.get(&cache.cache[y].into()) else {
                continue;
            };
            if let Ok((_, tile_terrain_info, options)) = tiles.get_mut(tile_entity) {
                if convert_tile(
                    id,
                    &player_marker.id(),
                    cache.cache[y].into(),
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
            continue;
        }
    }
}
