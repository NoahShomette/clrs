mod nuke;

use crate::buildings::Activate;
use bevy::prelude::{
    Commands, Component, Entity, FromReflect, Query, Reflect, Res, ResMut, Time, Timer, TimerMode,
    With, Without,
};
use bevy_ecs_tilemap::prelude::TileStorage;
use bevy_ggf::game_core::state::{Changed, DespawnedObjects};
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::{Object, ObjectGridPosition, ObjectId};
use bevy_ggf::player::PlayerMarker;

pub fn destroy_buildings(
    abilities: Query<
        (
            Entity,
            &PlayerMarker,
            &ObjectId,
            &ObjectGridPosition,
            &AbilityMarker,
        ),
        (With<Object>, With<DestroyAbility>),
    >,
    mut tiles: Query<(Entity, &PlayerMarker), (Without<Object>, With<Tile>)>,
    mut tile_storage_query: Query<(&MapId, &TileStorage)>,
    mut commands: Commands,
    mut despawn_objects: ResMut<DespawnedObjects>,
) {
    for (building_entity, player_marker, object_id, object_grid_pos, ability) in abilities.iter() {
        let Some((_, tile_storage)) = tile_storage_query
            .iter_mut()
            .find(|(id, _)| id == &&MapId{ id: 1 })else {
            continue;
        };

        let tile_entity = tile_storage.get(&object_grid_pos.tile_position).unwrap();

        let Ok((entity, tile_marker)) = tiles.get_mut(tile_entity) else {
            continue;
        };

        if player_marker != tile_marker && ability.requires_player_territory {
            println!("killing abilities");
            despawn_objects
                .despawned_objects
                .insert(*object_id, Changed::default());
            commands.entity(building_entity).despawn();
        }
    }
}

pub fn update_ability_timers(
    mut timers: Query<(Entity, &mut AbilityCooldown), Without<Activate>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut timer) in timers.iter_mut() {
        timer.timer.tick(time.delta());
        if timer.timer.finished() {
            commands.entity(entity).insert(Activate);
            timer.timer = Timer::from_seconds(timer.timer_reset, TimerMode::Once);
            timer.timer_ticks = timer.timer_ticks.saturating_sub(1);
        }
    }
}

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct DestroyAbility;

#[derive(Default, Clone, Copy, Eq, Hash, Debug, PartialEq, Reflect, FromReflect)]
pub enum Abilities {
    #[default]
    Nuke,
    Sacrifice,
    Boost,
}

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct AbilityMarker {
    pub requires_player_territory: bool,
}

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct Ability<T> {
    pub ability_type: T,
}

#[derive(Default, Clone, Debug, Component, Reflect, FromReflect)]
pub struct AbilityCooldown {
    pub timer: Timer,
    pub timer_ticks: u32,
    pub timer_reset: f32,
}
