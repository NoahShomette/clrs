use crate::abilities::{Ability, AbilityCooldown, DestroyAbility};
use crate::buildings::building_pathfinding::PathfindStrengthExt;
use crate::buildings::{get_neighbors_tilepos, Activate, Simulate};
use crate::color_system::{ColorConflictGuarantees, ConflictType, TileColor};
use crate::objects::ObjectCachedMap;
use bevy::prelude::{
    Commands, Component, Entity, EventWriter, FromReflect, Query, Reflect, With, Without,
};
use bevy_ecs_tilemap::prelude::*;
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::{ObjectGridPosition, ObjectId};
use bevy_ggf::player::PlayerMarker;
use rand::Rng;

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct Expand {
    pub strength: u32,
    pub min_tile_strengthen: u32,
    pub max_tile_strengthen: u32,
}

impl PathfindStrengthExt for Expand {
    fn pathfinding_strength(&self) -> u32 {
        self.strength
    }
}

// two parts - we pulse outwards, checking the outside neighbors of each tile. If the outside neighbors
// are not the same player then we damage their color by one. Otherwise at that point we stop.
pub fn simulate_expand_from_cache(
    mut tile_storage_query: Query<(Entity, &MapId, &TileStorage, &TilemapSize)>,
    mut tiles: Query<
        (
            Entity,
            &TileTerrainInfo,
            Option<(&mut PlayerMarker, &mut TileColor)>,
        ),
        (With<Tile>, Without<Ability<Expand>>, Without<MapId>),
    >,
    expands: Query<
        (
            Entity,
            &ObjectGridPosition,
            &ObjectId,
            &PlayerMarker,
            &Ability<Expand>,
            &AbilityCooldown,
            &ObjectCachedMap,
        ),
        (Without<MapId>, With<Activate>, With<Simulate>),
    >,
    mut event_writer: EventWriter<ColorConflictGuarantees>,
    mut commands: Commands,
) {
    let Some((_, _, tile_storage, tilemap_size)) = tile_storage_query
        .iter_mut()
        .find(|(_, id, _, _)| id == &&MapId { id: 1 })
    else {
        return;
    };

    for (entity, object_grid_position, _, player_marker, expand, ability_cooldown, cache) in
        expands.iter()
    {
        commands.entity(entity).remove::<Activate>();
        let mut rng = rand::thread_rng();

        let rndm = rng.gen_range(
            expand.ability_type.min_tile_strengthen..=expand.ability_type.max_tile_strengthen,
        );
        for _ in 0..rndm {
            event_writer.send(ColorConflictGuarantees {
                tile_pos: Into::<TilePos>::into(object_grid_position.tile_position),
                casting_player: player_marker.id(),
                affect_casting_player: true,
                affect_neutral: true,
                affect_other_players: true,
                conflict_type: ConflictType::Natural,
            });
        }

        let mut target_tiles = vec![];

        'main_loop: for tile in cache.cache.iter() {
            let mut neighbors = get_neighbors_tilepos(Into::<TilePos>::into(*tile), tilemap_size);
            neighbors.push(Into::<TilePos>::into(*tile));

            for neighbor in neighbors.iter() {
                let Some(tile_entity) = tile_storage.get(&Into::<TilePos>::into(*neighbor)) else {
                    continue;
                };
                if let Ok((_, _, options)) = tiles.get_mut(tile_entity) {
                    if let Some((tile_player_marker, _)) = options.as_ref() {
                        if player_marker.id() == tile_player_marker.id() {
                            target_tiles.push(tile);
                            continue 'main_loop;
                        }
                    }
                }
            }
        }

        for tile in target_tiles.iter() {
            let rndm = rng.gen_range(
                expand.ability_type.min_tile_strengthen..=expand.ability_type.max_tile_strengthen,
            );

            for _ in 0..rndm {
                event_writer.send(ColorConflictGuarantees {
                    tile_pos: Into::<TilePos>::into(**tile),
                    casting_player: player_marker.id(),
                    affect_casting_player: true,
                    affect_neutral: true,
                    affect_other_players: true,
                    conflict_type: ConflictType::Natural,
                });
            }
        }

        if ability_cooldown.timer_ticks == 0 {
            commands.entity(entity).insert(DestroyAbility);
            commands.entity(entity).remove::<Activate>();
        } else {
            commands.entity(entity).remove::<Activate>();
        }
    }
}
