use bevy::ecs::system::SystemState;
use bevy::prelude::{Color, Entity, Mut, Query, ReflectComponent, Without, World};
use bevy_ascii_terminal::{Terminal, TileFormatter};
use bevy_ecs_tilemap::prelude::TilePos;
use bevy_ggf::game_core::state::{DespawnedObjects, ObjectState, PlayerState, TileState};
use bevy_ggf::game_core::Game;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::object::ObjectId;
use bevy_ggf::player::Player;

pub fn update_game_state(world: &mut World) {
    world.resource_scope(|mut world, mut game: Mut<Game>| {
        let game_state = game.get_state_diff(0);
        //println!("{:?}", game_state);
        let registration = game.type_registry.read();

        let tiles: Vec<TileState> = game_state.tiles.into_iter().collect();

        for tile in tiles {
            let mut system_state: SystemState<Query<(Entity, &TilePos, &Tile), Without<ObjectId>>> =
                SystemState::new(&mut world);
            let mut tile_query = system_state.get_mut(&mut world);

            if let Some((entity, _, _)) = tile_query
                .iter_mut()
                .find(|(_, id, _)| id == &&tile.tile_pos)
            {
                world.entity_mut(entity).despawn();
                let entity = world.spawn_empty().id();
                for component in tile.components.into_iter() {
                    let type_info = component.type_name();
                    if let Some(type_registration) = registration.get_with_name(type_info) {
                        if let Some(reflect_component) =
                            type_registration.data::<ReflectComponent>()
                        {
                            reflect_component.insert(&mut world.entity_mut(entity), &*component);
                        }
                    }
                }
            } else {
                let mut entity = world.spawn_empty();
                for component in tile.components {
                    let type_info = component.type_name();
                    if let Some(type_registration) = registration.get_with_name(type_info) {
                        if let Some(reflect_component) =
                            type_registration.data::<ReflectComponent>()
                        {
                            reflect_component.insert(&mut entity, &*component);
                        }
                    } else {
                    }
                }
            }
        }

        let objects: Vec<ObjectState> = game_state.objects.into_iter().collect();
        for object in objects {
            let mut system_state: SystemState<Query<(Entity, &ObjectId)>> =
                SystemState::new(&mut world);

            let mut object_query = system_state.get(&mut world);

            if let Some((entity, object_id)) = object_query
                .iter_mut()
                .find(|(_, id)| id == &&object.object_id)
            {
                world.entity_mut(entity).despawn();
                let entity = world.spawn_empty().id();
                for component in object.components {
                    let type_info = component.type_name();
                    if let Some(type_registration) = registration.get_with_name(type_info) {
                        if let Some(reflect_component) =
                            type_registration.data::<ReflectComponent>()
                        {
                            reflect_component.insert(&mut world.entity_mut(entity), &*component);
                        }
                    }
                }
            } else {
                let entity = world.spawn_empty().id();

                for component in object.components {
                    let type_info = component.type_name();
                    if let Some(type_registration) = registration.get_with_name(type_info) {
                        if let Some(reflect_component) =
                            type_registration.data::<ReflectComponent>()
                        {
                            reflect_component.insert(&mut world.entity_mut(entity), &*component);
                        }
                    }
                }
            }
        }

        let objects: Vec<PlayerState> = game_state.players.into_iter().collect();
        for player in objects {
            let mut system_state: SystemState<Query<(Entity, &Player)>> =
                SystemState::new(&mut world);

            let mut object_query = system_state.get(&mut world);

            if let Some((entity, _)) = object_query
                .iter_mut()
                .find(|(_, id)| id == &&player.player_id)
            {
                world.entity_mut(entity).despawn();
                let entity = world.spawn_empty().id();
                for component in player.components {
                    let type_info = component.type_name();
                    if let Some(type_registration) = registration.get_with_name(type_info) {
                        if let Some(reflect_component) =
                            type_registration.data::<ReflectComponent>()
                        {
                            reflect_component.insert(&mut world.entity_mut(entity), &*component);
                        }
                    }
                }
            } else {
                let entity = world.spawn_empty().id();
                for component in player.components {
                    let type_info = component.type_name();
                    if let Some(type_registration) = registration.get_with_name(type_info) {
                        if let Some(reflect_component) =
                            type_registration.data::<ReflectComponent>()
                        {
                            reflect_component.insert(&mut world.entity_mut(entity), &*component);
                        }
                    }
                }
            }
        }

        let objects: Vec<ObjectId> = game_state.despawned_objects.into_iter().collect();
        for object in objects {
            let mut system_state: SystemState<Query<(Entity, &ObjectId)>> =
                SystemState::new(&mut world);

            let mut object_query = system_state.get(&mut world);

            if let Some((entity, object_id)) =
                object_query.iter_mut().find(|(_, id)| id == &&object)
            {
                world.entity_mut(entity).despawn();
            }
        }

        drop(registration);
        game.clear_changed();
    });
}
