use crate::audio::ObjectSpawnedSound;
use crate::color_system::{TileColor, TileColorStrength};
use crate::draw::{DrawObject, DrawTile, UpdateTile};
use crate::mapping::MapTileIndex;
use crate::objects::ObjectIndex;
use bevy::ecs::system::{Command, Commands, ResMut};
use bevy::ecs::world::Mut;
use bevy::log::info_span;
use bevy::prelude::{
    Component, DespawnRecursiveExt, Entity, Query, TransformBundle, VisibilityBundle, Without,
};
use bevy_ecs_tilemap::prelude::TilePos;
use bevy_ggf::game_core::saving::ComponentBinaryState;
use bevy_ggf::game_core::state::{ObjectState, PlayerState, TileState};
use bevy_ggf::game_core::Game;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::object::ObjectId;
use bevy_ggf::player::{Player, PlayerMarker};

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

pub fn update_main_world_game_state(
    mut tile_query: Query<
        (
            Entity,
            &TilePos,
            &Tile,
            Option<&TileColor>,
            Option<&PlayerMarker>,
        ),
        Without<ObjectId>,
    >,
    mut object_query: Query<(Entity, &ObjectId)>,
    mut player_query: Query<(Entity, &Player)>,
    mut game: ResMut<Game>,
    mut commands: Commands,
    mut map_tile_index: ResMut<MapTileIndex>,
    mut object_index: ResMut<ObjectIndex>,
) {
    let pp_loop = info_span!("Getting game state diff", name = "game_state_diff").entered();
    let game_state = game.get_state_diff(0);
    pp_loop.exit();

    //println!("{:?}", game_state);

    let pp_loop = info_span!("Updating tiles", name = "updating tiles").entered();

    let tiles: Vec<TileState> = game_state.tiles.into_iter().collect();

    for tile in tiles {
        if let Some(entity) = map_tile_index.hashmap.get(&tile.tile_pos) {
            if let Ok((entity, _, _, option_tile_color, option_player_marker)) =
                tile_query.get(*entity)
            {
                let mut old_object_state = OldTileState {
                    tile_color: Default::default(),
                    player_id: None,
                };

                if let Some(tile_color) = option_tile_color {
                    old_object_state.tile_color = Some(tile_color.clone());
                }
                if let Some(player_marker) = option_player_marker {
                    old_object_state.player_id = Some(player_marker.id().clone());
                }

                commands
                    .entity(entity)
                    .insert((UpdateTile, old_object_state))
                    .remove::<TileColor>()
                    .remove::<PlayerMarker>();

                for component in tile.components.into_iter() {
                    commands.add(InsertComponentBinaryStateCommand {
                        component_binary_state: component,
                        entity,
                    });
                }
            }
        } else {
            let entity = commands.spawn_empty().id();
            for component in tile.components.into_iter() {
                commands.add(InsertComponentBinaryStateCommand {
                    component_binary_state: component,
                    entity,
                });
            }
            map_tile_index.hashmap.insert(tile.tile_pos, entity);
            commands.entity(entity).insert((
                Tile,
                UpdateTile,
                DrawTile,
                VisibilityBundle::default(),
                TransformBundle::default(),
                tile.tile_pos,
            ));
        }
    }
    pp_loop.exit();
    let pp_loop = info_span!("Updating objects", name = "updating objects").entered();

    let objects: Vec<ObjectState> = game_state.objects.into_iter().collect();
    for object in objects {
        let entity = if let Some(entity) = object_index.hashmap.get(&object.object_id) {
            if let Ok((entity, _)) = object_query.get(*entity) {
                entity
            } else {
                continue;
            }
        } else {
            let entity = commands
                .spawn((
                    object.object_id,
                    object.object_grid_position,
                    ObjectSpawnedSound,
                    DrawObject,
                    VisibilityBundle::default(),
                    TransformBundle::default(),
                ))
                .id();
            object_index.hashmap.insert(object.object_id, entity);
            entity
        };

        for component in object.components {
            commands.add(InsertComponentBinaryStateCommand {
                component_binary_state: component,
                entity,
            });
        }
    }
    pp_loop.exit();

    let pp_loop = info_span!("Updating players", name = "updating players").entered();

    let players: Vec<PlayerState> = game_state.players.into_iter().collect();
    for player in players {
        if let Some((entity, _)) = player_query
            .iter_mut()
            .find(|(_, id)| id == &&player.player_id)
        {
            commands.entity(entity).despawn_recursive();
        }
        let entity = commands.spawn(player.player_id).id();
        for component in player.components {
            commands.add(InsertComponentBinaryStateCommand {
                component_binary_state: component,
                entity,
            });
        }
    }
    pp_loop.exit();

    let pp_loop = info_span!("despawning objects", name = "despawning objects").entered();

    let despawned_objects: Vec<ObjectId> = game_state.despawned_objects.into_iter().collect();
    for object in despawned_objects {
        if let Some((entity, _)) = object_query.iter_mut().find(|(_, id)| id == &&object) {
            commands.entity(entity).despawn_recursive();
        }
    }
    pp_loop.exit();

    game.clear_changed();
}

pub struct InsertComponentBinaryStateCommand {
    component_binary_state: ComponentBinaryState,
    entity: Entity,
}

impl Command for InsertComponentBinaryStateCommand {
    fn write(self, world: &mut bevy::prelude::World) {
        world.resource_scope(|world: &mut bevy::prelude::World, game: Mut<Game>| {
            let Some(mut entity_mut) = world.get_entity_mut(self.entity) else {
                return;
            };

            game.component_registry
                .deserialize_component_onto(&self.component_binary_state, &mut entity_mut);
        });
    }
}
