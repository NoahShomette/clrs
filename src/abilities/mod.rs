pub mod expand;
pub mod fortify;
pub mod nuke;

use crate::buildings::Activate;
use bevy::prelude::{
    Commands, Component, Entity, FromReflect, Query, Reflect, Res, ResMut, Time, Timer, TimerMode,
    With, Without,
};
use bevy_ggf::game_core::state::{Changed, DespawnedObjects};
use bevy_ggf::object::{Object, ObjectId};

pub fn destroy_abilities(
    abilities: Query<(Entity, &ObjectId, &AbilityMarker), (With<Object>, With<DestroyAbility>)>,
    mut commands: Commands,
    mut despawn_objects: ResMut<DespawnedObjects>,
) {
    for (building_entity, object_id, ability) in abilities.iter() {
        despawn_objects
            .despawned_objects
            .insert(*object_id, Changed::default());
        commands.entity(building_entity).despawn();
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
    Fortify,
    Expand,
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
