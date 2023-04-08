use crate::player::PlayerPoints;
use bevy::app::{App, Plugin};
use bevy::prelude::{
    Color, Commands, Component, Entity, EventReader, EventWriter, FromReflect, Mut, Query, ResMut,
    Resource, With, Without,
};
use bevy::reflect::Reflect;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::TileStorage;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ggf::game_core::state::Changed;
use bevy_ggf::mapping::terrain::TileTerrainInfo;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::mapping::MapId;
use bevy_ggf::object::ObjectId;
use bevy_ggf::player::{Player, PlayerMarker};
use rand::{thread_rng, Rng};

pub struct ColorSystemPlugin;

impl Plugin for ColorSystemPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
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

/// Function that will take the tile query and the player and see if - returns whether the checked
/// tile is the given players team
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
            }
            return true;
        } else {
            event_writer.send(ColorConflictEvent {
                from_object: *from_object,
                tile_pos,
                player: *player,
            });
        }
    } else {
        event_writer.send(ColorConflictEvent {
            from_object: *from_object,
            tile_pos,
            player: *player,
        });
    }
    return false;
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
) {
    for (tile_pos, player_id_vec) in color_conflicts.conflicts.iter() {
        let mut id_hashmap: HashMap<usize, u32> = HashMap::default();
        let mut objects: Vec<usize> = vec![];
        for id in player_id_vec.iter() {
            if objects.contains(&id.1) {
                continue;
            }
            objects.push(id.1);
            let count = id_hashmap.entry(id.0).or_insert(0);
            let count = *count;
            id_hashmap.insert(id.0, count.saturating_add(1));
        }

        let mut handle_conflicts = true;

        while handle_conflicts {
            if id_hashmap.is_empty() {
                handle_conflicts = false;
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
                .find(|(id, _)| id == &&MapId{ id: 1 })else {
                handle_conflicts = false;
                continue;
            };

            let tile_entity = tile_storage.get(&tile_pos).unwrap();

            let Ok((entity, _, options)) = tiles.get_mut(tile_entity) else {
                handle_conflicts = false;
                continue;
            };

            match options {
                None => {
                    commands.entity(entity).insert((
                        TileColor {
                            tile_color_strength: TileColorStrength::One,
                        },
                        PlayerMarker::new(highest.0),
                        Changed::default(),
                    ));
                    for (entity, mut player_points, player_id) in player_query.iter_mut() {
                        if player_id.id() == highest.0 {
                            increase_ability_points(entity, &mut player_points, &mut commands);
                        }
                    }
                    handle_conflicts = false;
                }
                Some((tile_player_marker, mut tile_color)) => {
                    if highest.0 == tile_player_marker.id() {
                        if let TileColorStrength::Five = tile_color.tile_color_strength {
                            //id_hashmap.remove(&highest.0);
                            handle_conflicts = false;
                        } else {
                            tile_color.strengthen();
                            commands.entity(entity).insert(Changed::default());
                            handle_conflicts = false;
                        }
                    } else {
                        tile_color.damage();
                        commands.entity(entity).insert(Changed::default());
                        if let TileColorStrength::Neutral = tile_color.tile_color_strength {
                            commands.entity(entity).remove::<PlayerMarker>();
                            commands.entity(entity).remove::<TileColor>();
                        }
                        handle_conflicts = false;
                    }
                }
            }
        }
    }
    color_conflicts.conflicts.clear();
}

pub fn increase_building_points(
    player_points_entity: Entity,
    mut player_points: &mut PlayerPoints,
    commands: &mut Commands,
) {
    if player_points.building_points < 50 {
        player_points.building_points = player_points.building_points.saturating_add(1);
        commands
            .entity(player_points_entity)
            .insert(Changed::default());
        return;
    }
    let mut rng = thread_rng();
    let amount_fifty_points: f64 = player_points.building_points as f64 / 50.0;
    let chance = rng.gen_bool((amount_fifty_points - 0.0) / (4.0 - 0.0));
    if !chance {
        player_points.building_points = player_points.building_points.saturating_add(1);
        commands
            .entity(player_points_entity)
            .insert(Changed::default());
    }
}

pub fn increase_ability_points(
    player_points_entity: Entity,
    mut player_points: &mut PlayerPoints,
    commands: &mut Commands,
) {
    if player_points.ability_points < 50 {
        player_points.ability_points = player_points.ability_points.saturating_add(1);
        commands
            .entity(player_points_entity)
            .insert(Changed::default());
        return;
    }
    let mut rng = thread_rng();
    let amount_fifty_points: f64 = player_points.ability_points as f64 / 50.0;
    let chance = rng.gen_bool((amount_fifty_points - 0.0) / (3.0 - 0.0));
    if !chance {
        player_points.ability_points = player_points.ability_points.saturating_add(1);
        commands
            .entity(player_points_entity)
            .insert(Changed::default());
    }
}

pub fn handle_color_conflict_guarantees(
    mut color_conflicts: ResMut<ColorConflicts>,
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
) {
    for (tile_pos, conflict_info) in color_conflicts.guaranteed_conflicts.iter() {
        for (casting_player, affect_casting_player, affect_neutral, affect_other_players, conflict_type) in
            conflict_info.iter()
        {
            let Some((_, tile_storage)) = tile_storage_query
                .iter_mut()
                .find(|(id, _)| id == &&MapId{ id: 1 })else {
                continue;
            };

            let tile_entity = tile_storage.get(&tile_pos).unwrap();

            let Ok((entity, _, options)) = tiles.get_mut(tile_entity) else {
                continue;
            };

            match options {
                None => {
                    if *affect_neutral && ConflictType::Damage != *conflict_type {
                        commands.entity(entity).insert((
                            TileColor {
                                tile_color_strength: TileColorStrength::One,
                            },
                            PlayerMarker::new(*casting_player),
                            Changed::default(),
                        ));
                    }
                }
                Some((tile_player_marker, mut tile_color)) => {
                    if *casting_player == tile_player_marker.id() && *affect_casting_player {
                        match conflict_type {
                            ConflictType::Damage => {
                                tile_color.damage();
                                commands.entity(entity).insert(Changed::default());
                                if let TileColorStrength::Neutral = tile_color.tile_color_strength {
                                    commands.entity(entity).remove::<PlayerMarker>();
                                    commands.entity(entity).remove::<TileColor>();
                                }
                            }
                            _ => {
                                if let TileColorStrength::Five = tile_color.tile_color_strength {
                                } else {
                                    tile_color.strengthen();
                                    commands.entity(entity).insert(Changed::default());
                                }
                            }
                        }
                    } else if *affect_other_players {
                        match conflict_type {
                            ConflictType::Damage => {
                                tile_color.damage();
                                commands.entity(entity).insert(Changed::default());
                                if let TileColorStrength::Neutral = tile_color.tile_color_strength {
                                    commands.entity(entity).remove::<PlayerMarker>();
                                    commands.entity(entity).remove::<TileColor>();
                                }
                            }
                            ConflictType::Stengthen => {
                                if let TileColorStrength::Five = tile_color.tile_color_strength {
                                } else {
                                    tile_color.strengthen();
                                    commands.entity(entity).insert(Changed::default());
                                }
                            }
                            ConflictType::Natural => {
                                tile_color.damage();
                                commands.entity(entity).insert(Changed::default());
                                if let TileColorStrength::Neutral = tile_color.tile_color_strength {
                                    commands.entity(entity).remove::<PlayerMarker>();
                                    commands.entity(entity).remove::<TileColor>();
                                }
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

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Reflect, FromReflect)]
pub enum TileColorStrength {
    #[default]
    Neutral,
    One,
    Two,
    Three,
    Four,
    Five,
}

#[derive(Default, Clone, Eq, Hash, Debug, PartialEq, Component, Reflect, FromReflect)]
pub struct TileColor {
    pub tile_color_strength: TileColorStrength,
}

impl TileColor {
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

pub enum PlayerColors {
    Blue,
    Red,
    Green,
    Purple,
}

impl PlayerColors {
    pub fn get_color(player_id: usize) -> Color {
        return match player_id {
            0 => Color::BLUE,
            1 => Color::RED,
            2 => Color::GREEN,
            _ => Color::INDIGO,
        };
    }
    pub fn get_colors_from(&mut self) -> Color {
        return match self {
            PlayerColors::Blue => Color::BLUE,
            PlayerColors::Red => Color::RED,
            PlayerColors::Green => Color::GREEN,
            PlayerColors::Purple => Color::INDIGO,
        };
    }
}
