use crate::actions::Actions;
use crate::buildings::BuildingTypes;
use crate::color_system::ColorConflicts;
use crate::game::GameData;
use crate::player::PlayerPoints;
use bevy::prelude::{Entity, Query, Res, ResMut, With};
use bevy_ecs_tilemap::prelude::{TileColor, TilePos, TileStorage};
use bevy_ggf::mapping::tiles::{ObjectStackingClass, Tile, TileObjectStacks};
use bevy_ggf::mapping::MapId;
use bevy_ggf::player::{Player, PlayerMarker};
use rand::{thread_rng, Rng};

pub fn run_ai(
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
) {
    let Some((_, tile_storage)) = tile_storage_query
        .iter_mut()
        .find(|(id, _)| id == &&MapId{ id: 1 })else {
        return;
    };

    for (entity, mut player_points, player, mut actions) in player_query.iter_mut() {
        actions.tile_pos = None;
        actions.target_world_pos = false;
        actions.try_place_ability = false;
        actions.try_place_building = false;
        actions.placed_building = false;
        actions.placed_ability = false;

        if player_points.building_points < 50 {
            continue;
        }

        let mut sorted_highest_conflicts: Vec<(TilePos, usize)> = vec![];
        for (tile_pos, player_id_vec) in color_conflicts.conflicts.iter().filter(|value| {
            let tile_entity = tile_storage.get(&value.0).unwrap();
            let Ok((entity, _, tile_object_stacks,  options)) = tiles.get_mut(tile_entity) else {
                return false;
            };

            if !tile_object_stacks.has_space(&ObjectStackingClass {
                stack_class: game_data.stacking_classes.get("Building").unwrap().clone(),
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
            let mut rng = thread_rng();
            match info.1 {
                0..=1 => {
                    let chance = rng.gen_bool(0.5);
                    actions.selected_building = match chance {
                        true => BuildingTypes::Line,
                        false => BuildingTypes::Pulser,
                    }
                }
                2..=4 => {
                    let chance = rng.gen_range(0..=2);
                    actions.selected_building = match chance {
                        0 => BuildingTypes::Scatter,
                        1 => BuildingTypes::Line,
                        _ => BuildingTypes::Pulser,
                    }
                }
                _ => {
                    let chance = rng.gen_bool(0.7);
                    actions.selected_building = match chance {
                        true => BuildingTypes::Scatter,
                        false => BuildingTypes::Pulser,
                    }
                }
            }

            actions.try_place_building = true;
            actions.tile_pos = Some(info.0);
        }
    }
}
