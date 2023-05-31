use crate::color_system::{TileColor, TileColorStrength};
use crate::draw::{DrawObject, DrawTile};
use bevy::ecs::system::SystemState;
use bevy::ecs::world::EntityMut;
use bevy::prelude::{
    Component, ComputedVisibility, DespawnRecursiveExt, Entity, FromReflect, GlobalTransform, Mut,
    Query, ReflectComponent, Transform, TransformBundle, VisibilityBundle, Without, World,
};
use bevy_ecs_tilemap::prelude::TilePos;
use bevy_ggf::game_core::state::{DespawnedObjects, ObjectState, PlayerState, TileState};
use bevy_ggf::game_core::Game;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::object::ObjectId;
use bevy_ggf::player::{Player, PlayerMarker};
use crate::audio::ObjectSpawnedSound;

#[derive(Component)]
pub struct OldObjectState {
    pub tile_color_strength: Option<TileColorStrength>,
}

impl Default for OldObjectState {
    fn default() -> Self {
        OldObjectState {
            tile_color_strength: None,
        }
    }
}

#[derive(Component)]
pub struct OldTileState {
    pub tile_color: Option<TileColor>,
    pub player_id: Option<usize>,
}

impl Default for OldTileState {
    fn default() -> Self {
        OldTileState {
            tile_color: None,
            player_id: None,
        }
    }
}

pub fn update_game_state(world: &mut World) {
    world.resource_scope(|mut world, mut game: Mut<Game>| {
        let game_state = game.get_state_diff(0);
        let registration = game.type_registry.read();

        let tiles: Vec<TileState> = game_state.tiles.into_iter().collect();

        for tile in tiles {
            let mut system_state: SystemState<Query<(Entity, &TilePos, &Tile), Without<ObjectId>>> =
                SystemState::new(&mut world);
            let mut tile_query = system_state.get_mut(&mut world);

            let mut old_object_state = OldTileState {
                tile_color: Default::default(),
                player_id: None,
            };

            if let Some((entity, _, _)) = tile_query
                .iter_mut()
                .find(|(_, id, _)| id == &&tile.tile_pos)
            {
                if let Some(tile_color) = world.get::<TileColor>(entity) {
                    old_object_state.tile_color = Some(tile_color.clone());
                }
                if let Some(player_marker) = world.get::<PlayerMarker>(entity) {
                    old_object_state.player_id = Some(player_marker.id().clone());
                }
                match world.get_entity_mut(entity) {
                    None => {}
                    Some(entity_mut) => {
                        entity_mut.despawn_recursive();
                    }
                }
            }
            let entity = world.spawn_empty().id();
            for component in tile.components.into_iter() {
                let type_info = component.type_name();
                if let Some(type_registration) = registration.get_with_name(type_info) {
                    if let Some(reflect_component) = type_registration.data::<ReflectComponent>() {
                        reflect_component.insert(&mut world.entity_mut(entity), &*component);
                    }
                }
            }
            world.entity_mut(entity).insert(old_object_state);
            world.entity_mut(entity).insert(DrawTile);
            world.entity_mut(entity).insert(VisibilityBundle::default());
            world.entity_mut(entity).insert(TransformBundle::default());
        }

        let objects: Vec<ObjectState> = game_state.objects.into_iter().collect();
        for object in objects {
            let mut system_state: SystemState<Query<(Entity, &ObjectId)>> =
                SystemState::new(&mut world);

            let entity = world.spawn_empty().id();

            let mut object_query = system_state.get(&mut world);
            if let Some((entity, object_id)) = object_query
                .iter_mut()
                .find(|(_, id)| id == &&object.object_id)
            {
                world.entity_mut(entity).despawn_recursive();
            } else {
                world.entity_mut(entity).insert(ObjectSpawnedSound);
            }

            for component in object.components {
                let type_info = component.type_name();
                if let Some(type_registration) = registration.get_with_name(type_info) {
                    if let Some(reflect_component) = type_registration.data::<ReflectComponent>() {
                        reflect_component.insert(&mut world.entity_mut(entity), &*component);
                    }
                }
            }
            world.entity_mut(entity).insert(DrawObject);
            world.entity_mut(entity).insert(VisibilityBundle::default());
            world.entity_mut(entity).insert(TransformBundle::default());
        }

        let players: Vec<PlayerState> = game_state.players.into_iter().collect();
        for player in players {
            let mut system_state: SystemState<Query<(Entity, &Player)>> =
                SystemState::new(&mut world);

            let mut object_query = system_state.get(&mut world);

            if let Some((entity, _)) = object_query
                .iter_mut()
                .find(|(_, id)| id == &&player.player_id)
            {
                world.entity_mut(entity).despawn_recursive();
            }
            let entity = world.spawn_empty().id();
            for component in player.components {
                let type_info = component.type_name();
                if let Some(type_registration) = registration.get_with_name(type_info) {
                    if let Some(reflect_component) = type_registration.data::<ReflectComponent>() {
                        reflect_component.insert(&mut world.entity_mut(entity), &*component);
                    }
                }
            }
        }

        let despawned_objects: Vec<ObjectId> = game_state.despawned_objects.into_iter().collect();
        for object in despawned_objects {
            let mut system_state: SystemState<Query<(Entity, &ObjectId)>> =
                SystemState::new(&mut world);

            let mut object_query = system_state.get(&mut world);

            if let Some((entity, object_id)) =
                object_query.iter_mut().find(|(_, id)| id == &&object)
            {
                world.entity_mut(entity).despawn_recursive();
            }
        }

        drop(registration);
        game.clear_changed();
    });
}
