use crate::color_system::{increase_building_points, TileColor, TileColorStrength};
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;
use bevy_ggf::mapping::tiles::Tile;
use bevy_ggf::player::{Player, PlayerMarker};
use ns_defaults::camera::CursorWorldPos;
use std::time::Duration;

pub struct PlayerPlugin;

#[derive(
    Component, Reflect, FromReflect, Default, Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd,
)]
#[reflect(Component)]
pub struct PlayerPoints {
    pub building_points: u32,
    pub ability_points: u32,
}

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {}
}

pub fn update_player_points(
    tile_query: Query<(&Tile, &PlayerMarker, &TileColor)>,
    mut player_query: Query<(Entity, &mut PlayerPoints, &Player)>,
    mut points_timer: Local<Timer>,
    time: Res<Time>,
    mut commands: Commands,
) {
    points_timer.tick(time.delta());
    if points_timer.finished() {
        let mut player_points_hashmap: HashMap<usize, u32> = HashMap::new();
        for (_, tile_marker, tile_color) in tile_query.iter() {
            if let TileColorStrength::Five = tile_color.tile_color_strength {
                let count = player_points_hashmap.entry(tile_marker.id()).or_insert(0);
                let count = *count;
                player_points_hashmap.insert(tile_marker.id(), count.saturating_add(1));
            }
        }

        for (entity, mut player_points, player_id) in player_query.iter_mut() {
            let points = *player_points_hashmap.entry(player_id.id()).or_insert(0) / 16;
            for _ in 0..points {
                increase_building_points(entity, &mut player_points, &mut commands);
            }
        }
        points_timer.set_duration(Duration::from_secs_f32(1.0));
        points_timer.reset();
    }
}

/// We added the ns_default camera setup which handles this. Leaving this in case we want to remove
/// camera movement. We would need to reinsert CursorWorldPos though
fn update_cursor_world_pos(
    mut query: Query<(&GlobalTransform, &Camera)>,
    mut cursor_world_pos: ResMut<CursorWorldPos>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok((global_transform, camera)) = query.get_single_mut() else{
        return;
    };

    let Ok(wnd) = windows.get_single() else {
        return;
    };

    //if the cursor is inside the current window then we want to update the cursor position
    if let Some(current_cursor_position) = wnd.cursor_position() {
        let Some(ray) = camera
            .viewport_to_world(global_transform, current_cursor_position) else{
            return;
        };
        cursor_world_pos.cursor_world_pos = ray.origin.truncate();
    }
}
