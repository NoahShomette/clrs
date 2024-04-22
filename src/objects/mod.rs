use std::collections::BTreeMap;

use bevy::{
    app::Plugin,
    ecs::{
        component::Component,
        entity::Entity,
        query::{Added, With, Without},
        system::{Query, ResMut, Resource, SystemState},
        world::{Mut, World},
    },
    utils::HashMap,
};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ggf::{
    game_core::change_detection::DespawnObject,
    mapping::{object, tiles::TilePosition, MapId},
    movement::{TileMoveCheckMeta, TileMoveChecks},
    object::ObjectId,
    pathfinding::{dijkstra::Node, PathfindAlgorithm, PathfindMap},
};

use crate::{
    buildings::Building,
    color_system::ColorConflictCallback,
    pathfinding::{
        AddObjectToTileToObjectIndex, IsColorableNodeCheck, RemoveObjectFromTileToObjectIndex,
    },
};

pub struct ObjectsPlugin;

impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<ObjectIndex>();
    }
}

#[derive(Resource, Default)]
pub struct ObjectIndex {
    pub hashmap: HashMap<ObjectId, Entity>,
}

#[derive(Default, Clone, Eq, Debug, PartialEq, Component, serde::Serialize, serde::Deserialize)]
pub struct ObjectCachedMap {
    pub cache: Vec<TilePosition>,
}

/// A Resource that contains a mapping from [`TilePos`] to every object that can affect that tile.
/// When simulating the game, any tiles that changed will access the objects that can affect it and make sure that the objects are simulated
#[derive(Resource, Default)]
pub struct TileToObjectIndex {
    pub map: HashMap<TilePos, Vec<ObjectId>>,
}

pub fn update_objects_index(
    mut object_index: ResMut<ObjectIndex>,
    obejcts_query: Query<(Entity, &ObjectId), Added<ObjectId>>,
) {
    for (entity, object_id) in obejcts_query.iter() {
        object_index.hashmap.insert(object_id.clone(), entity);
    }
}

// two parts - we pulse outwards, checking the outside neighbors of each tile. If the outside neighbors
// are not the same player then we damage their color by one. Otherwise at that point we stop.
pub fn simulate_simple_building_cache<
    Type: Component,
    Pathfinder: PathfindAlgorithm<TilePos, Node, Building<Type>> + Default,
    PathfinderMap: PathfindMap<TilePos, Node, Pathfinder::PathfindOutput, Building<Type>>
        + Default
        + AddObjectToTileToObjectIndex,
>(
    world: &mut World,
) {
    world.resource_scope(
        |mut world: &mut World, mut tile_to_object_index: Mut<TileToObjectIndex>| {
            let mut system_state: SystemState<
                Query<
                    (Entity, &ObjectId),
                    (
                        Without<MapId>,
                        Without<ObjectCachedMap>,
                        With<Building<Type>>,
                    ),
                >,
            > = SystemState::new(&mut world);
            let pulsers = system_state.get_mut(&mut world);

            let mut pathfind = Pathfinder::default();

            let mut tile_move_checks = TileMoveChecks {
                tile_move_checks: vec![TileMoveCheckMeta {
                    check: Box::new(IsColorableNodeCheck),
                }],
            };

            let pulsers: Vec<(Entity, ObjectId)> = pulsers
                .into_iter()
                .map(|(entity, object_id)| (entity, object_id.clone()))
                .collect();

            for (entity, object_id) in pulsers {
                let mut pathfind_map = PathfinderMap::default();

                pathfind.pathfind(
                    MapId { id: 1 },
                    entity,
                    &mut world,
                    &mut tile_move_checks,
                    &mut None::<ColorConflictCallback>,
                    &mut pathfind_map,
                );

                let mut cache_component = ObjectCachedMap { cache: vec![] };

                let mut btree_cache = BTreeMap::new();

                pathfind_map.add_to_index(object_id, &mut tile_to_object_index, &mut btree_cache);

                for vec in btree_cache.iter_mut() {
                    let mut converted = vec
                        .1
                        .iter()
                        .map(|x| Into::<TilePosition>::into(*x))
                        .collect();
                    cache_component.cache.append(&mut converted);
                }

                world.entity_mut(entity).insert(cache_component);
            }

            system_state.apply(&mut world);
        },
    );
}

// two parts - we pulse outwards, checking the outside neighbors of each tile. If the outside neighbors
// are not the same player then we damage their color by one. Otherwise at that point we stop.
pub fn delete_building_from_tile_index_cache<
    Type: Component,
    Pathfinder: PathfindAlgorithm<TilePos, Node, Building<Type>> + Default,
    PathfinderMap: PathfindMap<TilePos, Node, Pathfinder::PathfindOutput, Building<Type>>
        + Default
        + RemoveObjectFromTileToObjectIndex,
>(
    world: &mut World,
) {
    world.resource_scope(
        |mut world: &mut World, mut tile_to_object_index: Mut<TileToObjectIndex>| {
            let mut system_state: SystemState<
                Query<
                    (Entity, &ObjectId),
                    (Without<MapId>, With<Building<Type>>, With<DespawnObject>),
                >,
            > = SystemState::new(&mut world);
            let pulsers = system_state.get_mut(&mut world);

            let mut pathfind = Pathfinder::default();

            let mut tile_move_checks = TileMoveChecks {
                tile_move_checks: vec![TileMoveCheckMeta {
                    check: Box::new(IsColorableNodeCheck),
                }],
            };

            let pulsers: Vec<(Entity, ObjectId)> = pulsers
                .into_iter()
                .map(|(entity, object_id)| (entity, object_id.clone()))
                .collect();

            for (entity, object_id) in pulsers {
                let mut pathfind_map = PathfinderMap::default();

                pathfind.pathfind(
                    MapId { id: 1 },
                    entity,
                    &mut world,
                    &mut tile_move_checks,
                    &mut None::<ColorConflictCallback>,
                    &mut pathfind_map,
                );
                pathfind_map.remove_from_index(object_id, &mut tile_to_object_index);
            }

            system_state.apply(&mut world);
        },
    );
}
