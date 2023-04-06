use bevy::ecs::system::SystemState;
use bevy::prelude::{Commands, Entity, Mut, Query, Reflect, World};
use bevy_ecs_tilemap::prelude::*;
use bevy_ggf::game_core::command::{GameCommand, GameCommands};
use bevy_ggf::mapping::{Map, MapDeSpawned, MapId, MapIdProvider, MapSpawned, SpawnRandomMap};
use bevy_ggf::mapping::terrain::{TerrainType, TileTerrainInfo};
use bevy_ggf::mapping::tiles::{BggfTileBundle, BggfTileObjectBundle, Tile, TileObjects, TileObjectStacks};
use bevy_ggf::movement::TerrainMovementCosts;
use bevy_ggf::player::PlayerList;
use crate::color_system::TileColorStrength;

pub trait MapCommandsExt {
    fn spawn_testing_map(
        &mut self,
        tile_map_size: TilemapSize,
        tilemap_type: TilemapType,
        tilemap_tile_size: TilemapTileSize,
        map_terrain_vec: Vec<TerrainType>,
        tile_stack_rules: TileObjectStacks,
    ) -> SpawnTestingMap;
}

impl MapCommandsExt for GameCommands {
    fn spawn_testing_map(
        &mut self,
        tile_map_size: TilemapSize,
        tilemap_type: TilemapType,
        tilemap_tile_size: TilemapTileSize,
        map_terrain_type_vec: Vec<TerrainType>,
        tile_stack_rules: TileObjectStacks,
    ) -> SpawnTestingMap {
        self.queue.push(SpawnTestingMap {
            tile_map_size,
            tilemap_type,
            tilemap_tile_size,
            map_terrain_type_vec: map_terrain_type_vec.clone(),
            tile_stack_rules: tile_stack_rules.clone(),
            spawned_map_id: None,
        });
        SpawnTestingMap {
            tile_map_size,
            tilemap_type,
            tilemap_tile_size,
            map_terrain_type_vec,
            tile_stack_rules,
            spawned_map_id: None,
        }
    }
}

#[derive(Clone, Reflect)]
pub struct SpawnTestingMap {
    tile_map_size: TilemapSize,
    tilemap_type: TilemapType,
    tilemap_tile_size: TilemapTileSize,
    map_terrain_type_vec: Vec<TerrainType>,
    tile_stack_rules: TileObjectStacks,
    spawned_map_id: Option<MapId>,
}

impl GameCommand for SpawnTestingMap {
    fn execute(&mut self, world: &mut World) -> Result<(), String> {
        let map_size = self.tile_map_size;
        let mut tile_storage = TileStorage::empty(map_size);
        let tilemap_type = self.tilemap_type;
        let tilemap_entity = world.spawn_empty().id();
        
        

        world.resource_scope(|world, terrain_movement_costs: Mut<TerrainMovementCosts>| {
            for x in 0..map_size.x {
                for y in 0..map_size.y {
                    let tile_pos = TilePos { x, y };

                    let tile_entity = world
                        .spawn(BggfTileBundle {
                            tile: Tile,
                            tile_terrain_info: TileTerrainInfo {
                                terrain_type: self.map_terrain_type_vec[0].clone(),
                            },
                            tile_pos,
                            tilemap_id: TilemapId(tilemap_entity),
                        })
                        .insert(BggfTileObjectBundle {
                            tile_stack_rules: self.tile_stack_rules.clone(),
                            tile_objects: TileObjects::default(),
                        })
                        .insert(bevy_ggf::game_core::state::Changed::default())
                        .insert(TileColor::default())
                        .id();

                    tile_storage.set(&tile_pos, tile_entity);
                }
            }
        });

        let tile_size = self.tilemap_tile_size;
        let grid_size: TilemapGridSize = tile_size.into();
        let map_type = TilemapType::default();

        // If we have already spawned this map in then just use that
        let id = self.spawned_map_id.unwrap_or_else(|| {
            let mut map_id_provider = world.resource_mut::<MapIdProvider>();
            map_id_provider.next_id_component()
        });

        //world.send_event::<MapSpawned>(MapSpawned { map_id: id });

        world
            .entity_mut(tilemap_entity)
            .insert((grid_size, map_type, map_size, tile_storage, tile_size))
            .insert(Map {
                tilemap_type,
                map_size,
                tilemap_entity,
            })
            .insert(id);

        self.spawned_map_id = Some(id);

        Ok(())
    }
}
