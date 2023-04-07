pub mod line;
pub mod pulser;
pub mod scatter;

use crate::buildings::line::Line;
use crate::buildings::pulser::Pulser;
use crate::buildings::scatter::Scatters;
use crate::game::GameData;
use bevy::prelude::{
    Bundle, Commands, Component, Entity, FromReflect, Query, Reflect, Res, ResMut, Timer, With,
    Without,
};
use bevy::time::{Time, TimerMode};
use bevy::utils::hashbrown::HashMap;
use bevy_ecs_tilemap::prelude::{TileStorage, TilemapSize};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ggf::game_core::state::{Changed, DespawnedObjects};
use bevy_ggf::mapping::tiles::{ObjectStackingClass, Tile};
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::{Object, ObjectGridPosition, ObjectId, ObjectInfo};
use bevy_ggf::player::PlayerMarker;

pub fn destroy_buildings(
    buildings: Query<
        (
            Entity,
            &PlayerMarker,
            &ObjectId,
            &ObjectGridPosition,
            &BuildingMarker,
        ),
        With<Object>,
    >,
    mut tiles: Query<(Entity, &PlayerMarker), (Without<Object>, With<Tile>)>,
    mut tile_storage_query: Query<(&MapId, &TileStorage)>,
    mut commands: Commands,
    mut despawn_objects: ResMut<DespawnedObjects>,
) {
    for (building_entity, player_marker, object_id, object_grid_pos, building) in buildings.iter() {
        let Some((_, tile_storage)) = tile_storage_query
            .iter_mut()
            .find(|(id, _)| id == &&MapId{ id: 1 })else {
            continue;
        };

        let tile_entity = tile_storage.get(&object_grid_pos.tile_position).unwrap();

        let Ok((entity, tile_marker)) = tiles.get_mut(tile_entity) else {
            continue;
        };

        if player_marker != tile_marker {
            println!("killing buildings");
            despawn_objects
                .despawned_objects
                .insert(*object_id, Changed::default());
            commands.entity(building_entity).despawn();
        }
    }
}

pub fn update_building_timers(
    mut timers: Query<(Entity, &mut BuildingCooldown), Without<Activate>>,
    mut commands: Commands,
    mut time: Res<Time>,
) {
    for (entity, mut timer) in timers.iter_mut() {
        timer.timer.tick(time.delta());
        if timer.timer.finished() {
            commands.entity(entity).insert(Activate);
            timer.timer = Timer::from_seconds(timer.timer_reset, TimerMode::Once);
        }
    }
}

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct BuildingMarker;

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct Building<T> {
    pub building_type: T,
}

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct Activate;

#[derive(Default, Clone, Debug, Component, Reflect, FromReflect)]
pub struct BuildingCooldown {
    pub timer: Timer,
    pub timer_reset: f32,
}

#[derive(Default, Clone, Copy, Eq, Hash, Debug, PartialEq, Reflect, FromReflect)]
pub enum BuildingTypes {
    #[default]
    Pulser,
    Scatter,
    Line,
}

#[derive(Default, Clone, Copy, Eq, Hash, Debug, PartialEq)]
pub struct TileNode {
    cost: Option<u32>,
    prior_node: TilePos,
    tile_pos: TilePos,
}

pub fn get_neighbors_tilepos(
    node_to_get_neighbors: TilePos,
    tilemap_size: &TilemapSize,
) -> Vec<TilePos> {
    let mut neighbor_tiles: Vec<TilePos> = vec![];
    let origin_tile = node_to_get_neighbors;
    if let Some(north) =
        TilePos::from_i32_pair(origin_tile.x as i32, origin_tile.y as i32 + 1, tilemap_size)
    {
        neighbor_tiles.push(north);
    }
    if let Some(east) =
        TilePos::from_i32_pair(origin_tile.x as i32 + 1, origin_tile.y as i32, tilemap_size)
    {
        neighbor_tiles.push(east);
    }
    if let Some(south) =
        TilePos::from_i32_pair(origin_tile.x as i32, origin_tile.y as i32 - 1, tilemap_size)
    {
        neighbor_tiles.push(south);
    }
    if let Some(west) =
        TilePos::from_i32_pair(origin_tile.x as i32 - 1, origin_tile.y as i32, tilemap_size)
    {
        neighbor_tiles.push(west);
    }

    neighbor_tiles
}

fn tile_cost_check(
    max_cost: u32,
    tile_pos: &TilePos,
    move_from_tile_pos: &TilePos,
    tile_nodes: &mut HashMap<TilePos, TileNode>,
) -> bool {
    let Some((tile_node, move_from_tile_node)) = get_two_node_mut(tile_nodes, *tile_pos, *move_from_tile_pos) else {
        return false;
    };

    return if tile_node.cost.is_some() {
        if move_from_tile_node.cost.unwrap() + 1 < (tile_node.cost.unwrap()) {
            tile_node.cost = Some(move_from_tile_node.cost.unwrap() + 1);
            tile_node.prior_node = move_from_tile_node.tile_pos;
            true
        } else {
            false
        }
    } else if move_from_tile_node.cost.unwrap() + 1 <= max_cost {
        tile_node.cost = Some(move_from_tile_node.cost.unwrap() + 1);
        tile_node.prior_node = move_from_tile_node.tile_pos;
        true
    } else {
        false
    };
}

/// Returns a mutable reference for both nodes specified and returns them in the same order
fn get_two_node_mut(
    tiles: &mut HashMap<TilePos, TileNode>,
    node_one: TilePos,
    node_two: TilePos,
) -> Option<(&mut TileNode, &mut TileNode)> {
    // either get the current item in the move nodes or make a new default node and add it to the hashmap and then return that
    return if let Some(nodes) = tiles.get_many_mut([&node_one, &node_two]) {
        match nodes {
            [node1, node2] => {
                if node1.tile_pos == node_one {
                    Some((node1, node2))
                } else {
                    Some((node2, node1))
                }
            }
        }
    } else {
        None
    };
}
