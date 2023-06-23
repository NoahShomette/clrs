use crate::buildings::pulser::Pulser;
use crate::buildings::Building;
use crate::color_system::{TileColor, TileColorStrength};
use bevy::ecs::system::SystemState;
use bevy::prelude::{Entity, Query, With, Without, World};
use bevy::utils::hashbrown::HashMap;
use bevy_ecs_tilemap::prelude::{TilePos, TilemapSize};
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::mapping::MapId;
use bevy_ggf::movement::{ObjectMovement, TileMoveCheck, TileMovementCosts};
use bevy_ggf::object::{ObjectGridPosition, ObjectId};
use bevy_ggf::player::PlayerMarker;

pub fn check_if_tile_is_colorable(
    world: &mut World,
    tile_entity: Entity,
    player_id: usize,
) -> bool {
    let mut system_state: SystemState<
        Query<(&TileTerrainInfo, Option<(&PlayerMarker, &TileColor)>), (With<Tile>)>,
    > = SystemState::new(world);
    let tiles = system_state.get(world);
    if let Ok((tile_terrain_info, options)) = tiles.get(tile_entity) {
        if tile_terrain_info.terrain_type.terrain_class.name != String::from("Colorable") {
            return false;
        }
        return if let Some((player_marker, tile_color)) = options.as_ref() {
            if player_marker.id() == player_id
                && TileColorStrength::Five == tile_color.tile_color_strength
            {
                return false;
            }
            true
        } else {
            true
        }
    }
    false
}

pub struct IsColorableNodeCheck;

impl TileMoveCheck for IsColorableNodeCheck {
    fn is_valid_move(
        &self,
        _entity_moving: Entity,
        tile_entity: Entity,
        _tile_pos: &TilePos,
        _last_tile_pos: &TilePos,
        world: &mut World,
    ) -> bool {
        let mut system_state: SystemState<Query<&TileTerrainInfo>> = SystemState::new(world);
        let mut tile_query = system_state.get_mut(world);

        let Ok(tile_terrain_info) = tile_query.get(tile_entity) else{
            return false
        };

        tile_terrain_info.terrain_type.name == String::from("BasicColorable")
    }
}

pub struct NodeIsPlayersCheck;

impl TileMoveCheck for NodeIsPlayersCheck {
    fn is_valid_move(
        &self,
        moving_entity: Entity,
        tile_entity: Entity,
        _checking_tile_pos: &TilePos,
        _move_from_tile_pos: &TilePos,
        world: &mut World,
    ) -> bool {
        let mut system_state: SystemState<(
            Query<(Entity, &ObjectId, &PlayerMarker)>,
            Query<(&TileTerrainInfo, Option<&PlayerMarker>, Option<&TileColor>)>,
        )> = SystemState::new(world);
        let (mut object_query, mut tile_query) = system_state.get_mut(world);
        let Ok((entity, object_id, player_marker)) = object_query.get(moving_entity) else{
            return false
        };

        let Ok((tile_terrain_info, tile_player_marker, tile_color)) = tile_query.get(tile_entity) else{
            return false
        };

        if tile_player_marker.is_some() && tile_color.is_some() {
            if player_marker.id() == tile_player_marker.unwrap().id() {
                return true;
            }
        }
        return false;
    }
}
