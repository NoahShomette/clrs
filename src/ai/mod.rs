use crate::abilities::Abilities;
use crate::actions::Actions;
use crate::buildings::BuildingTypes;
use crate::color_system::{ColorConflicts, TileColor, TileColorStrength};
use crate::game::GameData;
use crate::player::PlayerPoints;
use bevy::prelude::{Commands, Entity, Query, Res, ResMut, With};
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage};
use bevy_ggf::game_core::state::Changed;
use bevy_ggf::mapping::terrain::{TerrainClass, TileTerrainInfo};
use bevy_ggf::mapping::tiles::{ObjectStackingClass, Tile, TileObjectStacks, TilePosition};
use bevy_ggf::mapping::MapId;
use bevy_ggf::player::{Player, PlayerMarker};
use rand::{thread_rng, Rng};

pub fn run_ai_building(
    color_conflicts: Res<ColorConflicts>,
    mut tiles: Query<
        (
            Entity,
            &TilePos,
            &TileObjectStacks,
            &TileTerrainInfo,
            Option<(&mut PlayerMarker, &mut TileColor)>,
        ),
        With<Tile>,
    >,
    mut tile_storage_query: Query<(&MapId, &TileStorage)>,
    mut player_query: Query<(Entity, &mut PlayerPoints, &Player, &mut Actions)>,
    game_data: Res<GameData>,
    mut commands: Commands,
) {
    let Some((_, tile_storage)) = tile_storage_query
        .iter_mut()
        .find(|(id, _)| id == &&MapId { id: 1 })
    else {
        return;
    };

    for (entity, mut player_points, player, mut actions) in player_query.iter_mut() {
        if player.id() == 0 {
            continue;
        }
        actions.building_tile_pos = None;
        actions.ability_tile_pos = None;
        actions.target_world_pos = false;
        actions.try_place_ability = false;
        actions.try_place_building = false;
        actions.placed_building = false;
        actions.placed_ability = false;

        if player_points.building_points < 50 {
            continue;
        }

        let mut rng = thread_rng();

        if !rng.gen_bool(0.80) {
            continue;
        }

        let mut low_health_tile_pos: Option<(TilePos, usize)> = None;

        let mut sorted_highest_conflicts: Vec<(TilePos, usize)> = vec![];
        for (tile_pos, player_id_vec) in color_conflicts.conflicts.iter().filter(|value| {
            let tile_entity = tile_storage.get(&value.0).unwrap();
            let Ok((entity, tile_pos, tile_object_stacks, tile_terrain_info, options)) =
                tiles.get_mut(tile_entity)
            else {
                return false;
            };

            if !tile_object_stacks.has_space(&ObjectStackingClass {
                stack_class: game_data.stacking_classes.get("Building").unwrap().clone(),
            }) {
                return false;
            }

            if let Some((player_marker, tile_color)) = options {
                if player_marker.id() == player.id() {
                    if low_health_tile_pos.is_none() {
                        low_health_tile_pos =
                            Some((*tile_pos, tile_color.get_number_representation() as usize));
                    } else {
                        let mut low_health_tile_pos = low_health_tile_pos.unwrap();
                        if tile_color.get_number_representation() < low_health_tile_pos.1 as u32 {
                            low_health_tile_pos.1 = tile_color.get_number_representation() as usize;
                            low_health_tile_pos.0 = *tile_pos;
                        }
                    }
                    return true;
                }
                return false;
            }
            return false;
        }) {
            let mut conflict_count: usize = 0;
            let mut objects: Vec<usize> = vec![];

            for (id, object_id) in player_id_vec.iter() {
                if objects.contains(&object_id) {
                    continue;
                }
                objects.push(*object_id);
                conflict_count = conflict_count + 1;
            }
            sorted_highest_conflicts.push((*tile_pos, conflict_count));
            sorted_highest_conflicts.sort_by(|a, b| a.1.cmp(&b.1));
        }

        let info: Option<(TilePos, usize)> = match sorted_highest_conflicts.get(0) {
            None => match low_health_tile_pos {
                None => {
                    let mut new_pos: Option<(TilePos, usize)> = None;
                    'emergency_tile_loop: for (
                        entity,
                        tile_pos,
                        tile_object_stacks,
                        tile_terrain_info,
                        options,
                    ) in tiles.iter()
                    {
                        if let Some((player_marker, tile_color)) = options {
                            if player_marker.id() == player.id() {
                                if !tile_object_stacks.has_space(&ObjectStackingClass {
                                    stack_class: game_data
                                        .stacking_classes
                                        .get("Building")
                                        .unwrap()
                                        .clone(),
                                }) {
                                    continue;
                                }

                                if check_if_neighbor_not_same(
                                    *tile_pos,
                                    player_marker.id(),
                                    &game_data,
                                    &tile_storage,
                                    &tiles,
                                ) {
                                    new_pos = Some((*tile_pos, 0));
                                    break 'emergency_tile_loop;
                                } else {
                                    continue;
                                }
                            }
                        }
                    }
                    new_pos
                }
                Some(tile_pos) => Some(tile_pos),
            },
            Some(info) => Some(*info),
        };
        let mut rng = thread_rng();

        if info.is_some() {
            match info.unwrap().1 {
                0..=0 => {
                    let chance = rng.gen_bool(0.5);
                    actions.selected_building = match chance {
                        true => BuildingTypes::Line,
                        false => BuildingTypes::Pulser,
                    }
                }
                1..=1 => {
                    let chance = rng.gen_range(0..=2);
                    actions.selected_building = match chance {
                        0 => BuildingTypes::Scatter,
                        1 => BuildingTypes::Line,
                        _ => BuildingTypes::Pulser,
                    }
                }
                _ => {
                    let chance = rng.gen_bool(0.6);
                    actions.selected_building = match chance {
                        true => BuildingTypes::Scatter,
                        false => BuildingTypes::Pulser,
                    }
                }
            }
            actions.try_place_building = true;
            actions.building_tile_pos = Some(info.unwrap().0.into());
            commands.entity(entity).insert(Changed::default());
        }
    }
}

pub fn run_ai_ability(
    color_conflicts: Res<ColorConflicts>,
    mut tiles: Query<
        (
            Entity,
            &TilePos,
            &TileObjectStacks,
            Option<(&mut PlayerMarker, &mut TileColor)>,
        ),
        With<Tile>,
    >,
    mut tile_storage_query: Query<(&MapId, &TileStorage)>,
    mut player_query: Query<(Entity, &mut PlayerPoints, &Player, &mut Actions)>,
    game_data: Res<GameData>,
    mut commands: Commands,
) {
    let Some((_, tile_storage)) = tile_storage_query
        .iter_mut()
        .find(|(id, _)| id == &&MapId { id: 1 })
    else {
        return;
    };

    for (entity, mut player_points, player, mut actions) in player_query.iter_mut() {
        if player.id() == 0 {
            continue;
        }
        if player_points.ability_points < 50 {
            continue;
        }

        let mut rng = thread_rng();

        if rng.gen_bool(0.3) {
            continue;
        }

        let ability_pick = rng.gen_range(0..=2);
        match ability_pick {
            //fortify
            0 => {
                let mut sorted_highest_conflicts: Vec<(TilePos, usize)> = vec![];
                for (tile_pos, player_id_vec) in color_conflicts.conflicts.iter().filter(|value| {
                    let tile_entity = tile_storage.get(&value.0).unwrap();
                    let Ok((entity, _, tile_object_stacks, options)) = tiles.get_mut(tile_entity)
                    else {
                        return false;
                    };

                    if !tile_object_stacks.has_space(&ObjectStackingClass {
                        stack_class: game_data.stacking_classes.get("Ability").unwrap().clone(),
                    }) {
                        return false;
                    }

                    if let Some((player_marker, tile_color)) = options {
                        return player_marker.id() == player.id();
                    }
                    return false;
                }) {
                    let mut conflict_count: usize = 0;
                    let mut objects: Vec<usize> = vec![];

                    for (id, object_id) in player_id_vec.iter() {
                        if objects.contains(&object_id) {
                            continue;
                        }
                        objects.push(*object_id);
                        conflict_count = conflict_count + 1;
                    }
                    sorted_highest_conflicts.push((*tile_pos, conflict_count));
                    sorted_highest_conflicts.sort_by(|a, b| a.1.cmp(&b.1));
                }
                if let Some(info) = sorted_highest_conflicts.get(0) {
                    actions.selected_ability = Abilities::Fortify;
                    actions.try_place_ability = true;
                    actions.ability_tile_pos = Some(info.0.into());
                }
            }
            // expand
            1 => {
                for (entity, tile_pos, tile_object_stacks, options) in tiles.iter() {
                    if options.is_none() {
                        actions.selected_ability = Abilities::Expand;
                        actions.try_place_ability = true;
                        actions.ability_tile_pos = Some(Into::<TilePosition>::into(*tile_pos));
                    }
                }
            }
            // nuke
            _ => {
                for (entity, tile_pos, tile_object_stacks, options) in tiles.iter() {
                    if options.is_some() {
                        let (player_marker, tile_color) = options.unwrap();
                        if player_marker.id() == player.id() {
                            continue;
                        }
                        actions.selected_ability = Abilities::Nuke;
                        actions.try_place_ability = true;
                        actions.ability_tile_pos = Some(Into::<TilePosition>::into(*tile_pos));
                    }
                }
            }
        }
        commands.entity(entity).insert(Changed::default());
    }
}

pub fn check_if_neighbor_not_same(
    target: TilePos,
    checking_player_id: usize,
    game_data: &Res<GameData>,
    tile_storage: &TileStorage,
    tile_query: &Query<
        (
            Entity,
            &TilePos,
            &TileObjectStacks,
            &TileTerrainInfo,
            Option<(&mut PlayerMarker, &mut TileColor)>,
        ),
        With<Tile>,
    >,
) -> bool {
    for count in 0..=3 {
        let tile_pos = match count {
            0 => TilePos {
                x: target.x.saturating_sub(1),
                y: target.y,
            },
            1 => TilePos {
                x: target.x,
                y: target.y + 1,
            },
            2 => TilePos {
                x: target.x + 1,
                y: target.y,
            },
            _ => TilePos {
                x: target.x,
                y: target.y.saturating_sub(1),
            },
        };
        if check_in_bounds(tile_pos, &game_data) {
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                let Ok((entity, tile_pos, tile_object_stacks, tile_terrain_info, options)) =
                    tile_query.get(tile_entity)
                else {
                    continue;
                };

                if tile_terrain_info.terrain_type.terrain_class
                    != (TerrainClass {
                        name: "Colorable".to_string(),
                    })
                {
                    continue;
                }

                if let Some((player_marker, tile_color)) = options {
                    if player_marker.id() != checking_player_id {
                        return true;
                    } else {
                        continue;
                    }
                } else {
                    return true;
                }
            } else {
                continue;
            }
        }
    }

    false
}

fn check_in_bounds(tile_pos: TilePos, game_data: &Res<GameData>) -> bool {
    if tile_pos.x > game_data.map_size_x - 1 {
        return false;
    }
    if tile_pos.y > game_data.map_size_y - 1 {
        return false;
    }
    true
}
