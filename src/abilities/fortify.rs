use crate::abilities::{Ability, AbilityCooldown, DestroyAbility};
use crate::buildings::building_pathfinding::PathfindStrengthExt;
use crate::buildings::{Activate, Simulate};
use crate::color_system::{ColorConflictGuarantees, ConflictType};
use crate::objects::ObjectCachedMap;
use bevy::prelude::{
    Commands, Component, Entity, EventWriter, FromReflect, Query, Reflect, With, Without,
};
use bevy_ecs_tilemap::prelude::*;
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::ObjectId;
use bevy_ggf::player::PlayerMarker;
use rand::Rng;

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct Fortify {
    pub strength: u32,
    pub min_tile_strengthen: u32,
    pub max_tile_strengthen: u32,
}

impl PathfindStrengthExt for Fortify {
    fn pathfinding_strength(&self) -> u32 {
        self.strength
    }
}

// two parts - we pulse outwards, checking the outside neighbors of each tile. If the outside neighbors
// are not the same player then we damage their color by one. Otherwise at that point we stop.
pub fn simulate_fortify_from_cache(
    pulsers: Query<
        (
            Entity,
            &ObjectId,
            &PlayerMarker,
            &Ability<Fortify>,
            &AbilityCooldown,
            &ObjectCachedMap,
        ),
        (Without<MapId>, With<Activate>, With<Simulate>),
    >,
    mut event_writer: EventWriter<ColorConflictGuarantees>,
    mut commands: Commands,
) {
    for (entity, _, player_marker, expand, ability_cooldown, cache) in pulsers.iter() {
        commands.entity(entity).remove::<Activate>();

        let mut rng = rand::thread_rng();
        for tile in cache.cache.iter() {
            let rndm = rng.gen_range(
                expand.ability_type.min_tile_strengthen..=expand.ability_type.max_tile_strengthen,
            );
            for _ in 0..rndm {
                event_writer.send(ColorConflictGuarantees {
                    tile_pos: Into::<TilePos>::into(*tile),
                    casting_player: player_marker.id(),
                    affect_casting_player: true,
                    affect_neutral: false,
                    affect_other_players: false,
                    conflict_type: ConflictType::Stengthen,
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
