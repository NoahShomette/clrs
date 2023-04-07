use crate::buildings::Building;
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

pub struct ColorSystemPlugin;

impl Plugin for ColorSystemPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

/// Function that will take the tile query and the player and see if - returns whether the checked
/// tile is the given players team
pub fn convert_tile(
    from_object: &ObjectId,
    player: &usize,
    tile_pos: TilePos,
    tile_terrain_info: &TileTerrainInfo,
    option: Option<(Mut<PlayerMarker>, Mut<TileColor>)>,
    event_writer: &mut EventWriter<ColorConflictEvent>,
) -> bool {
    if tile_terrain_info.terrain_type.terrain_class.name.as_str() != "Colorable" {
        return false;
    }

    if let Some((tile_player_marker, mut tile_color)) = option {
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
) {
    let events: Vec<ColorConflictEvent> = event_reader.into_iter().cloned().collect();
    for event in events {
        color_conflicts.register_conflict(event.tile_pos, event.player, event.from_object.id);
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
                            player_points.ability_points =
                                player_points.ability_points.saturating_add(1);
                            commands.entity(entity).insert(Changed::default());
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

#[derive(Default, Clone, Copy, Eq, Debug, PartialEq, Reflect, FromReflect)]
pub struct ColorConflictEvent {
    pub from_object: ObjectId,
    pub tile_pos: TilePos,
    pub player: usize,
}

#[derive(Default, Clone, Eq, Debug, PartialEq, Resource, Reflect, FromReflect)]
pub struct ColorConflicts {
    pub conflicts: HashMap<TilePos, Vec<(usize, usize)>>,
}

impl ColorConflicts {
    pub fn register_conflict(&mut self, tile_pos: TilePos, player: usize, from_object: usize) {
        if let Some(conflicts) = self.conflicts.get_mut(&tile_pos) {
            conflicts.push((player, from_object));
        } else {
            self.conflicts.insert(tile_pos, vec![(player, from_object)]);
        }
    }
}

pub enum ColorResult {}

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
    pub fn max_strength(&mut self) -> bool {
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
