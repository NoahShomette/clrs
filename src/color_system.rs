use crate::buildings::Simulate;
use crate::objects::{ObjectIndex, TileToObjectIndex};
use crate::player::PlayerPoints;
use bevy::app::{App, Plugin};
use bevy::ecs::system::SystemState;
use bevy::math::Vec3;
use bevy::prelude::{
    Commands, Component, Entity, EventReader, EventWriter, FromReflect, Mut, Query, ResMut,
    Resource, With, World,
};
use bevy::reflect::Reflect;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::TileStorage;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ggf::game_core::saving::{BinaryComponentId, SaveId};
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::ObjectId;
use bevy_ggf::pathfinding::PathfindCallback;
use bevy_ggf::player::{Player, PlayerMarker};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

pub struct ColorSystemPlugin;

impl Plugin for ColorSystemPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Default, Resource)]
pub struct PlayerTileChangedCount {
    pub player_gained_tiles: u32,
    pub player_lost_tiles: u32,
}

/// Function that will take the tile query and the player, register a guaranteed conflict for the tile,
/// and then check and return whether the checked tile is the given players team
pub fn register_guaranteed_color_conflict(
    player: &usize,
    affect_casting_player: bool,
    affect_neutral: bool,
    affect_other_players: bool,
    conflict_type: ConflictType,
    tile_pos: TilePos,
    tile_terrain_info: &TileTerrainInfo,
    option: &Option<(Mut<PlayerMarker>, Mut<TileColor>)>,
    event_writer: &mut EventWriter<ColorConflictGuarantees>,
) -> bool {
    if tile_terrain_info.terrain_type.terrain_class.name.as_str() != "Colorable" {
        return false;
    }

    if option.is_some() {
        let (tile_player_marker, tile_color) = option.as_ref().unwrap();

        if player == &tile_player_marker.id() && affect_casting_player {
            event_writer.send(ColorConflictGuarantees {
                tile_pos,
                casting_player: *player,
                affect_casting_player,
                affect_neutral,
                affect_other_players,
                conflict_type,
            });
            return true;
        } else if affect_other_players {
            event_writer.send(ColorConflictGuarantees {
                tile_pos,
                casting_player: *player,
                affect_casting_player,
                affect_neutral,
                affect_other_players,
                conflict_type,
            });
        }
    } else {
        event_writer.send(ColorConflictGuarantees {
            tile_pos,
            casting_player: *player,
            affect_casting_player,
            affect_neutral: true,
            affect_other_players,
            conflict_type,
        });
    }
    return false;
}

pub struct ColorConflictCallback;

#[derive(Resource)]
pub struct ColorConflictCallbackQueryState {
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
        EventWriter<'static, ColorConflictEvent>,
    )>,
}

impl PathfindCallback<TilePos> for ColorConflictCallback {
    fn foreach_tile(
        &mut self,
        pathfinding_entity: Entity,
        node_entity: Entity,
        node_pos: TilePos,
        node_cost: u32,
        world: &mut World,
    ) {
        let mut system_state = match world.remove_resource::<ColorConflictCallbackQueryState>() {
            None => {
                let system_state: SystemState<(
                    Query<(Entity, &ObjectId, &PlayerMarker)>,
                    Query<(&TileTerrainInfo, Option<&PlayerMarker>, Option<&TileColor>)>,
                    EventWriter<ColorConflictEvent>,
                )> = SystemState::new(world);
                ColorConflictCallbackQueryState {
                    query: system_state,
                }
            }
            Some(res) => res,
        };
        let (mut object_query, mut tile_query, mut event_writer) =
            system_state.query.get_mut(world);
        let Ok((entity, object_id, player_marker)) = object_query.get(pathfinding_entity) else {
            world.insert_resource(system_state);
            return;
        };

        let Ok((tile_terrain_info, tile_player_marker, tile_color)) = tile_query.get(node_entity)
        else {
            world.insert_resource(system_state);
            return;
        };

        if tile_terrain_info.terrain_type.name != String::from("BasicColorable") {
            world.insert_resource(system_state);
            return;
        }

        if tile_player_marker.is_some() && tile_color.is_some() {
            if player_marker.id() == tile_player_marker.unwrap().id() {
                if !tile_color.unwrap().max_strength() {
                    event_writer.send(ColorConflictEvent {
                        from_object: *object_id,
                        tile_pos: node_pos,
                        player: player_marker.id(),
                    });
                }
                world.insert_resource(system_state);
                return;
            } else {
                event_writer.send(ColorConflictEvent {
                    from_object: *object_id,
                    tile_pos: node_pos,
                    player: player_marker.id(),
                });
            }
        } else {
            event_writer.send(ColorConflictEvent {
                from_object: *object_id,
                tile_pos: node_pos,
                player: player_marker.id(),
            });
        }
        world.insert_resource(system_state);
        return;
    }
}

/// Function that will take the tile query and the player and see returns whether a tile was converted or not
pub fn convert_tile(
    from_object: &ObjectId,
    player: &usize,
    tile_pos: TilePos,
    tile_terrain_info: &TileTerrainInfo,
    option: &Option<(Mut<PlayerMarker>, Mut<TileColor>)>,
    event_writer: &mut EventWriter<ColorConflictEvent>,
) -> bool {
    if tile_terrain_info.terrain_type.terrain_class.name.as_str() != "Colorable" {
        return false;
    }

    if let Some((tile_player_marker, tile_color)) = option {
        if player == &tile_player_marker.id() {
            if !tile_color.max_strength() {
                event_writer.send(ColorConflictEvent {
                    from_object: *from_object,
                    tile_pos,
                    player: *player,
                });
                return true;
            }
            return false;
        } else {
            event_writer.send(ColorConflictEvent {
                from_object: *from_object,
                tile_pos,
                player: *player,
            });
            return true;
        }
    } else {
        event_writer.send(ColorConflictEvent {
            from_object: *from_object,
            tile_pos,
            player: *player,
        });
        return true;
    }
}

pub fn update_color_conflicts(
    mut color_conflicts: ResMut<ColorConflicts>,
    mut event_reader: EventReader<ColorConflictEvent>,
    mut guarantee_event_reader: EventReader<ColorConflictGuarantees>,
) {
    let events: Vec<ColorConflictEvent> = event_reader.into_iter().cloned().collect();
    for event in events {
        color_conflicts.register_conflict(event.tile_pos, event.player, event.from_object.id);
    }
    event_reader.clear();

    let events: Vec<ColorConflictGuarantees> =
        guarantee_event_reader.into_iter().cloned().collect();
    for event in events {
        color_conflicts.register_conflict_guarantee(
            event.tile_pos,
            event.casting_player,
            event.affect_neutral,
            event.affect_casting_player,
            event.affect_other_players,
            event.conflict_type,
        );
    }
    event_reader.clear();
}

pub fn handle_color_conflicts(
    mut color_conflicts: ResMut<ColorConflicts>,
    mut player_tiles_changed_count: ResMut<PlayerTileChangedCount>,
    mut commands: Commands,
    mut tiles: Query<
        (
            Entity,
            &TilePos,
            Option<(&mut PlayerMarker, &mut TileColor)>,
        ),
        With<Tile>,
    >,
    mut tile_storage_query: Query<(&MapId, &TileStorage)>,
    mut player_query: Query<(Entity, &mut PlayerPoints, &Player)>,
    tile_to_object_index: ResMut<TileToObjectIndex>,
    object_index: ResMut<ObjectIndex>,
    object_query: Query<(Entity, Option<&Simulate>), With<ObjectId>>,
) {
    player_tiles_changed_count.player_lost_tiles = 0;
    player_tiles_changed_count.player_gained_tiles = 0;

    for (tile_pos, player_id_vec) in color_conflicts.conflicts.iter() {
        let mut id_hashmap: HashMap<usize, u32> = HashMap::default();
        for (player_id, _object_id) in player_id_vec.iter() {
            let count = id_hashmap.entry(*player_id).or_insert(0);
            let count = *count;
            id_hashmap.insert(*player_id, count.saturating_add(1));
        }

        if id_hashmap.is_empty() {
            continue;
        }

        let mut highest: (usize, u32) = (0, 0);
        for (id, count) in id_hashmap.iter() {
            if count > &highest.1 {
                highest.0 = *id;
                highest.1 = *count;
            }
        }

        let Some((_, tile_storage)) = tile_storage_query
            .iter_mut()
            .find(|(id, _)| id == &&MapId { id: 1 })
        else {
            continue;
        };

        let tile_entity = tile_storage.get(&tile_pos).unwrap();

        let Ok((entity, _, options)) = tiles.get_mut(tile_entity) else {
            continue;
        };

        match options {
            None => {
                commands.entity(entity).insert((
                    TileColor {
                        tile_color_strength: TileColorStrength::One,
                    },
                    PlayerMarker::new(highest.0),
                ));
                for (entity, mut player_points, player_id) in player_query.iter_mut() {
                    if player_id.id() == 0 {
                        player_tiles_changed_count.player_gained_tiles = player_tiles_changed_count
                            .player_gained_tiles
                            .saturating_add(1);
                    }
                    if player_id.id() == highest.0 {
                        increase_ability_points(&mut player_points);
                    }
                }
            }
            Some((tile_player_marker, mut tile_color)) => {
                if highest.0 == tile_player_marker.id() {
                    if let TileColorStrength::Five = tile_color.tile_color_strength {
                        continue;
                    } else {
                        tile_color.strengthen();
                    }
                } else {
                    tile_color.damage();
                    if let TileColorStrength::Neutral = tile_color.tile_color_strength {
                        if tile_player_marker.id() == 0 {
                            player_tiles_changed_count.player_lost_tiles =
                                player_tiles_changed_count
                                    .player_lost_tiles
                                    .saturating_add(1);
                        }
                        commands.entity(entity).remove::<PlayerMarker>();
                        commands.entity(entity).remove::<TileColor>();
                    }
                }
            }
        }

        if let Some(object_vec) = tile_to_object_index.map.get(&tile_pos) {
            for object_id in object_vec.iter() {
                if let Some(entity) = object_index.hashmap.get(&object_id) {
                    if let Ok((entity, opt_simulate)) = object_query.get(*entity) {
                        if opt_simulate.is_none() {
                            commands.entity(entity).insert(Simulate);
                        }
                    }
                }
            }
        }
    }
    color_conflicts.conflicts.clear();
}

pub fn increase_building_points(mut player_points: &mut PlayerPoints) {
    if player_points.building_points < 50 {
        player_points.building_points = player_points.building_points.saturating_add(1);
        return;
    }
    let mut rng = thread_rng();
    let amount_fifty_points: f64 = player_points.building_points as f64 / 50.0;
    let chance = rng.gen_bool((amount_fifty_points - 0.0) / (4.0 - 0.0));
    if !chance {
        player_points.building_points = player_points.building_points.saturating_add(1);
    }
}

pub fn increase_ability_points(mut player_points: &mut PlayerPoints) {
    if player_points.ability_points < 50 {
        player_points.ability_points = player_points.ability_points.saturating_add(1);
        return;
    }
    let mut rng = thread_rng();
    let amount_fifty_points: f64 = player_points.ability_points as f64 / 50.0;
    let chance = rng.gen_bool((amount_fifty_points - 0.0) / (3.0 - 0.0));
    if !chance {
        player_points.ability_points = player_points.ability_points.saturating_add(1);
    }
}

pub fn handle_color_conflict_guarantees(
    mut color_conflicts: ResMut<ColorConflicts>,
    mut player_tiles_changed_count: ResMut<PlayerTileChangedCount>,
    mut commands: Commands,
    mut tiles: Query<
        (
            Entity,
            &TilePos,
            Option<(&mut PlayerMarker, &mut TileColor)>,
        ),
        With<Tile>,
    >,
    mut tile_storage_query: Query<(&MapId, &TileStorage)>,
    tile_to_object_index: ResMut<TileToObjectIndex>,
    object_index: ResMut<ObjectIndex>,
    object_query: Query<(Entity, Option<&Simulate>), With<ObjectId>>,
) {
    player_tiles_changed_count.player_lost_tiles = 0;
    player_tiles_changed_count.player_gained_tiles = 0;

    for (tile_pos, conflict_info) in color_conflicts.guaranteed_conflicts.iter() {
        for (
            casting_player,
            affect_casting_player,
            affect_neutral,
            affect_other_players,
            conflict_type,
        ) in conflict_info.iter()
        {
            let Some((_, tile_storage)) = tile_storage_query
                .iter_mut()
                .find(|(id, _)| id == &&MapId { id: 1 })
            else {
                continue;
            };

            let tile_entity = tile_storage.get(&tile_pos).unwrap();

            let Ok((entity, _, options)) = tiles.get_mut(tile_entity) else {
                continue;
            };

            match options {
                None => {
                    if *affect_neutral && ConflictType::Damage != *conflict_type {
                        if casting_player == &0 {
                            player_tiles_changed_count.player_gained_tiles =
                                player_tiles_changed_count
                                    .player_gained_tiles
                                    .saturating_add(1);
                        }
                        commands.entity(entity).insert((
                            TileColor {
                                tile_color_strength: TileColorStrength::One,
                            },
                            PlayerMarker::new(*casting_player),
                        ));
                    }
                }
                Some((tile_player_marker, mut tile_color)) => {
                    if *casting_player == tile_player_marker.id() && *affect_casting_player {
                        match conflict_type {
                            ConflictType::Damage => {
                                tile_color.damage();
                                if let TileColorStrength::Neutral = tile_color.tile_color_strength {
                                    player_tiles_changed_count.player_lost_tiles =
                                        player_tiles_changed_count
                                            .player_lost_tiles
                                            .saturating_add(1);

                                    commands.entity(entity).remove::<PlayerMarker>();
                                    commands.entity(entity).remove::<TileColor>();
                                }
                            }
                            _ => {
                                if let TileColorStrength::Five = tile_color.tile_color_strength {
                                } else {
                                    tile_color.strengthen();
                                }
                            }
                        }
                    } else if *affect_other_players {
                        match conflict_type {
                            ConflictType::Damage => {
                                tile_color.damage();
                                if let TileColorStrength::Neutral = tile_color.tile_color_strength {
                                    commands.entity(entity).remove::<PlayerMarker>();
                                    commands.entity(entity).remove::<TileColor>();
                                }
                            }
                            ConflictType::Stengthen => {
                                if let TileColorStrength::Five = tile_color.tile_color_strength {
                                } else {
                                    tile_color.strengthen();
                                }
                            }
                            ConflictType::Natural => {
                                tile_color.damage();
                                if let TileColorStrength::Neutral = tile_color.tile_color_strength {
                                    commands.entity(entity).remove::<PlayerMarker>();
                                    commands.entity(entity).remove::<TileColor>();
                                }
                            }
                        }
                    }
                }
            }
            if let Some(object_vec) = tile_to_object_index.map.get(&tile_pos) {
                for object_id in object_vec.iter() {
                    if let Some(entity) = object_index.hashmap.get(&object_id) {
                        if let Ok((entity, opt_simulate)) = object_query.get(*entity) {
                            if opt_simulate.is_none() {
                                commands.entity(entity).insert(Simulate);
                            }
                        }
                    }
                }
            }
        }
    }
    color_conflicts.guaranteed_conflicts.clear();
}

#[derive(Default, Clone, Copy, Eq, Debug, PartialEq, Reflect, FromReflect)]
pub struct ColorConflictEvent {
    pub from_object: ObjectId,
    pub tile_pos: TilePos,
    pub player: usize,
}

#[derive(Default, Clone, Copy, Eq, Debug, PartialEq, Reflect, FromReflect)]
pub struct ColorConflictGuarantees {
    pub tile_pos: TilePos,
    pub casting_player: usize,
    pub affect_casting_player: bool,
    pub affect_neutral: bool,
    pub affect_other_players: bool,
    pub conflict_type: ConflictType,
}

#[derive(Default, Clone, Copy, Eq, Debug, PartialEq, Reflect, FromReflect)]
pub enum ConflictType {
    #[default]
    Natural,
    Damage,
    Stengthen,
}

#[derive(Default, Clone, Eq, Debug, PartialEq, Resource, Reflect, FromReflect)]
pub struct ColorConflicts {
    pub conflicts: HashMap<TilePos, Vec<(usize, usize)>>,
    pub guaranteed_conflicts: HashMap<TilePos, Vec<(usize, bool, bool, bool, ConflictType)>>,
}

impl ColorConflicts {
    pub fn register_conflict(&mut self, tile_pos: TilePos, player: usize, from_object: usize) {
        if let Some(conflicts) = self.conflicts.get_mut(&tile_pos) {
            conflicts.push((player, from_object));
        } else {
            self.conflicts.insert(tile_pos, vec![(player, from_object)]);
        }
    }

    pub fn register_conflict_guarantee(
        &mut self,
        tile_pos: TilePos,
        casting_player: usize,
        affect_casting_player: bool,
        affect_neutral: bool,
        affect_other_players: bool,
        conflict_type: ConflictType,
    ) {
        if let Some(conflicts) = self.guaranteed_conflicts.get_mut(&tile_pos) {
            conflicts.push((
                casting_player,
                affect_casting_player,
                affect_neutral,
                affect_other_players,
                conflict_type,
            ));
        } else {
            self.guaranteed_conflicts.insert(
                tile_pos,
                vec![(
                    casting_player,
                    affect_casting_player,
                    affect_neutral,
                    affect_other_players,
                    conflict_type,
                )],
            );
        }
    }
}

#[derive(
    Default, Clone, Eq, Hash, Debug, PartialEq, Reflect, FromReflect, Serialize, Deserialize,
)]
pub enum TileColorStrength {
    #[default]
    Neutral,
    One,
    Two,
    Three,
    Four,
    Five,
}

#[derive(
    Default,
    Clone,
    Eq,
    Hash,
    Debug,
    PartialEq,
    Component,
    Reflect,
    FromReflect,
    Serialize,
    Deserialize,
)]
pub struct TileColor {
    pub tile_color_strength: TileColorStrength,
}

impl SaveId for TileColor {
    fn save_id(&self) -> BinaryComponentId {
        18
    }

    fn save_id_const() -> BinaryComponentId
    where
        Self: Sized,
    {
        18
    }

    #[doc = r" Serializes the state of the object at the given tick into binary. Only saves the keyframe and not the curve itself"]
    fn to_binary(&self) -> Option<Vec<u8>> {
        bincode::serialize(self).ok()
    }
}

impl TileColor {
    pub fn get_scale(&self) -> Vec3 {
        match self.tile_color_strength {
            TileColorStrength::Neutral => Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            TileColorStrength::One => Vec3 {
                x: 0.5,
                y: 0.5,
                z: 1.0,
            },
            TileColorStrength::Two => Vec3 {
                x: 0.6,
                y: 0.6,
                z: 1.0,
            },
            TileColorStrength::Three => Vec3 {
                x: 0.7,
                y: 0.7,
                z: 1.0,
            },
            TileColorStrength::Four => Vec3 {
                x: 0.8,
                y: 0.8,
                z: 1.0,
            },
            TileColorStrength::Five => Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        }
    }
    pub fn get_normalized_number_representation(&self) -> f32 {
        match self.tile_color_strength {
            TileColorStrength::Neutral => 0.0,
            TileColorStrength::One => 0.2,
            TileColorStrength::Two => 0.4,
            TileColorStrength::Three => 0.6,
            TileColorStrength::Four => 0.8,
            TileColorStrength::Five => 1.0,
        }
    }
    pub fn get_number_representation(&self) -> u32 {
        match self.tile_color_strength {
            TileColorStrength::Neutral => 0,
            TileColorStrength::One => 1,
            TileColorStrength::Two => 2,
            TileColorStrength::Three => 3,
            TileColorStrength::Four => 4,
            TileColorStrength::Five => 5,
        }
    }

    pub fn max_strength(&self) -> bool {
        return if self.tile_color_strength == TileColorStrength::Five {
            true
        } else {
            false
        };
    }

    pub fn damage(&mut self) {
        match self.tile_color_strength {
            TileColorStrength::Neutral => {}
            TileColorStrength::One => {
                self.tile_color_strength = TileColorStrength::Neutral;
            }
            TileColorStrength::Two => {
                self.tile_color_strength = TileColorStrength::One;
            }
            TileColorStrength::Three => {
                self.tile_color_strength = TileColorStrength::Two;
            }
            TileColorStrength::Four => {
                self.tile_color_strength = TileColorStrength::Three;
            }
            TileColorStrength::Five => {
                self.tile_color_strength = TileColorStrength::Four;
            }
        }
    }

    pub fn strengthen(&mut self) {
        match self.tile_color_strength {
            TileColorStrength::Neutral => {
                self.tile_color_strength = TileColorStrength::One;
            }
            TileColorStrength::One => {
                self.tile_color_strength = TileColorStrength::Two;
            }
            TileColorStrength::Two => {
                self.tile_color_strength = TileColorStrength::Three;
            }
            TileColorStrength::Three => {
                self.tile_color_strength = TileColorStrength::Four;
            }
            TileColorStrength::Four => {
                self.tile_color_strength = TileColorStrength::Five;
            }
            TileColorStrength::Five => {}
        }
    }
}
