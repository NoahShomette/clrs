use crate::buildings::pulser::{Pulser, PulserQueryState};
use crate::buildings::Building;
use crate::color_system::{
    ColorConflictCallbackQueryState, ColorConflictEvent, TileColor, TileColorStrength,
};
use bevy::ecs::system::SystemState;
use bevy::prelude::{Entity, EventWriter, Query, Resource, With, Without, World};
use bevy::utils::hashbrown::HashMap;
use bevy_ecs_tilemap::prelude::{TilePos, TilemapSize};
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::movement::TileMoveCheck;
use bevy_ggf::object::ObjectId;
use bevy_ggf::player::PlayerMarker;

#[derive(Resource)]
pub struct TileColorableCheckQueryState {
    pub query: SystemState<
        Query<
            'static,
            'static,
            (
                &'static TileTerrainInfo,
                Option<(&'static PlayerMarker, &'static TileColor)>,
            ),
            With<Tile>,
        >,
    >,
}

pub fn check_if_tile_is_colorable(
    world: &mut World,
    tile_entity: Entity,
    player_id: usize,
) -> bool {
    let mut system_state = match world.remove_resource::<TileColorableCheckQueryState>() {
        None => {
            let system_state: SystemState<
                Query<(&TileTerrainInfo, Option<(&PlayerMarker, &TileColor)>), With<Tile>>,
            > = SystemState::new(world);
            TileColorableCheckQueryState {
                query: system_state,
            }
        }
        Some(res) => res,
    };

    let tiles = system_state.query.get(world);
    if let Ok((tile_terrain_info, options)) = tiles.get(tile_entity) {
        if tile_terrain_info.terrain_type.terrain_class.name != String::from("Colorable") {
            world.insert_resource(system_state);
            return false;
        }
        return if let Some((player_marker, tile_color)) = options.as_ref() {
            if player_marker.id() == player_id
                && TileColorStrength::Five == tile_color.tile_color_strength
            {
                world.insert_resource(system_state);
                return false;
            }
            world.insert_resource(system_state);
            true
        } else {
            world.insert_resource(system_state);
            true
        };
    }
    world.insert_resource(system_state);
    false
}

#[derive(Resource)]
pub struct IsColorableNodeCheckQueryState {
    pub query: SystemState<Query<'static, 'static, &'static TileTerrainInfo>>,
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
        let mut system_state = match world.remove_resource::<IsColorableNodeCheckQueryState>() {
            None => {
                let system_state: SystemState<Query<&TileTerrainInfo>> = SystemState::new(world);
                IsColorableNodeCheckQueryState {
                    query: system_state,
                }
            }
            Some(res) => res,
        };
        let mut tile_query = system_state.query.get_mut(world);

        let Ok(tile_terrain_info) = tile_query.get(tile_entity) else{
            world.insert_resource(system_state);
            return false
        };
        let bool = tile_terrain_info.terrain_type.name == String::from("BasicColorable");
        world.insert_resource(system_state);
        bool
    }
}

#[derive(Resource)]
pub struct NodeIsPlayersCheckQueryState {
    pub query: SystemState<(
        Query<'static, 'static, (Entity, &'static ObjectId, &'static PlayerMarker)>,
        Query<
            'static,
            'static,
            (
                &'static TileTerrainInfo,
                Option<&'static PlayerMarker>,
                Option<&'static TileColor>,
            ),
        >,
    )>,
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
        let mut system_state = match world.remove_resource::<NodeIsPlayersCheckQueryState>() {
            None => {
                let system_state: SystemState<(
                    Query<(Entity, &ObjectId, &PlayerMarker)>,
                    Query<(&TileTerrainInfo, Option<&PlayerMarker>, Option<&TileColor>)>,
                )> = SystemState::new(world);
                NodeIsPlayersCheckQueryState {
                    query: system_state,
                }
            }
            Some(res) => res,
        };
        let (mut object_query, mut tile_query) = system_state.query.get_mut(world);
        let Ok((entity, object_id, player_marker)) = object_query.get(moving_entity) else{
            world.insert_resource(system_state);
            return false
        };

        let Ok((tile_terrain_info, tile_player_marker, tile_color)) = tile_query.get(tile_entity) else{
            world.insert_resource(system_state);
            return false
        };

        if tile_player_marker.is_some() && tile_color.is_some() {
            if player_marker.id() == tile_player_marker.unwrap().id() {
                world.insert_resource(system_state);
                return true;
            }
        }
        world.insert_resource(system_state);
        return false;
    }
}
