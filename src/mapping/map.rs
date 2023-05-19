use crate::level_loader::{Level, TileType};
use bevy::prelude::{Commands, Entity, Mut, Query, Reflect, World};
use bevy::utils::petgraph::visit::Walker;
use bevy_ecs_tilemap::prelude::*;
use bevy_ggf::game_core::command::{GameCommand, GameCommands};
use bevy_ggf::mapping::terrain::{TerrainType, TileTerrainInfo};
use bevy_ggf::mapping::tiles::{
    BggfTileBundle, BggfTileObjectBundle, Tile, TileObjectStacks, TileObjects,
};
use bevy_ggf::mapping::{Map, MapDeSpawned, MapId, MapIdProvider, MapSpawned};
use bevy_ggf::movement::TerrainMovementCosts;

pub trait MapCommandsExt {
    fn spawn_random_map(
        &mut self,
        tile_map_size: TilemapSize,
        map_terrain_vec: Vec<TerrainType>,
        tile_stack_rules: TileObjectStacks,
    ) -> SpawnRandomMap;

    fn spawn_map(
        &mut self,
        map_terrain_vec: Vec<TerrainType>,
        level_data: Level,
        colorable_tile_stack_rules: TileObjectStacks,
        non_colorable_tile_stack_rules: TileObjectStacks,
    ) -> SpawnMap;
}

impl MapCommandsExt for GameCommands {
    fn spawn_random_map(
        &mut self,
        tile_map_size: TilemapSize,
        map_terrain_type_vec: Vec<TerrainType>,
        tile_stack_rules: TileObjectStacks,
    ) -> SpawnRandomMap {
        self.queue.push(SpawnRandomMap {
            tile_map_size,
            map_terrain_type_vec: map_terrain_type_vec.clone(),
            tile_stack_rules: tile_stack_rules.clone(),
            spawned_map_id: None,
        });
        SpawnRandomMap {
            tile_map_size,
            map_terrain_type_vec,
            tile_stack_rules,
            spawned_map_id: None,
        }
    }

    fn spawn_map(
        &mut self,
        map_terrain_type_vec: Vec<TerrainType>,
        level_data: Level,
        colorable_tile_stack_rules: TileObjectStacks,
        non_colorable_tile_stack_rules: TileObjectStacks,
    ) -> SpawnMap {
        self.queue.push(SpawnMap {
            map_terrain_type_vec: map_terrain_type_vec.clone(),
            colorable_tile_stack_rules: colorable_tile_stack_rules.clone(),
            non_colorable_tile_stack_rules: non_colorable_tile_stack_rules.clone(),
            level_data: level_data.clone(),
            spawned_map_id: None,
        });
        SpawnMap {
            map_terrain_type_vec,
            colorable_tile_stack_rules: colorable_tile_stack_rules.clone(),
            non_colorable_tile_stack_rules,
            level_data: level_data.clone(),
            spawned_map_id: None,
        }
    }
}

#[derive(Clone, Reflect)]
pub struct SpawnRandomMap {
    tile_map_size: TilemapSize,
    map_terrain_type_vec: Vec<TerrainType>,
    tile_stack_rules: TileObjectStacks,
    spawned_map_id: Option<MapId>,
}

impl GameCommand for SpawnRandomMap {
    fn execute(&mut self, world: &mut World) -> Result<(), String> {
        let map_size = self.tile_map_size;
        let mut tile_storage = TileStorage::empty(map_size);
        let tilemap_type = TilemapType::Square;
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
        let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
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

#[derive(Clone, Reflect)]
pub struct SpawnMap {
    map_terrain_type_vec: Vec<TerrainType>,
    colorable_tile_stack_rules: TileObjectStacks,
    non_colorable_tile_stack_rules: TileObjectStacks,
    level_data: Level,
    spawned_map_id: Option<MapId>,
}

impl GameCommand for SpawnMap {
    fn execute(&mut self, world: &mut World) -> Result<(), String> {
        let map_size = TilemapSize {
            x: self.level_data.tiles.get(0).unwrap().len() as u32,
            y: self.level_data.tiles.len() as u32,
        };
        let mut tile_storage = TileStorage::empty(map_size);
        let tilemap_type = TilemapType::Square;
        let tilemap_entity = world.spawn_empty().id();

        world.resource_scope(|world, terrain_movement_costs: Mut<TerrainMovementCosts>| {
            for y in self.level_data.tiles.iter().enumerate() {
                for x in y.1.iter().enumerate() {
                    let tile_pos = TilePos {
                        x: x.0 as u32,
                        y: y.0 as u32,
                    };

                    let tile_entity = match x.1 {
                        TileType::Colorable => world
                            .spawn(BggfTileBundle {
                                tile: Tile,
                                tile_terrain_info: TileTerrainInfo {
                                    terrain_type: self.map_terrain_type_vec[0].clone(),
                                },
                                tile_pos,
                                tilemap_id: TilemapId(tilemap_entity),
                            })
                            .insert(BggfTileObjectBundle {
                                tile_stack_rules: self.colorable_tile_stack_rules.clone(),
                                tile_objects: TileObjects::default(),
                            })
                            .insert(bevy_ggf::game_core::state::Changed::default())
                            .insert(TileColor::default())
                            .id(),
                        TileType::NonColorable => world
                            .spawn(BggfTileBundle {
                                tile: Tile,
                                tile_terrain_info: TileTerrainInfo {
                                    terrain_type: self.map_terrain_type_vec[1].clone(),
                                },
                                tile_pos,
                                tilemap_id: TilemapId(tilemap_entity),
                            })
                            .insert(BggfTileObjectBundle {
                                tile_stack_rules: self.non_colorable_tile_stack_rules.clone(),
                                tile_objects: TileObjects::default(),
                            })
                            .insert(bevy_ggf::game_core::state::Changed::default())
                            .id(),
                    };

                    tile_storage.set(&tile_pos, tile_entity);
                }
            }
        });

        let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
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
