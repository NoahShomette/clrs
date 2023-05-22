use crate::draw::draw::{TILE_GAP, TILE_SIZE};
use crate::game::{start_game, GameData};
use crate::GameState;
use bevy::app::App;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::PrimaryWindow;
use ns_defaults::camera::{CursorWorldPos, GGFCamera2dBundle};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPos>();
        app.add_system(spawn_camera.in_schedule(OnEnter(GameState::Splash)));
        app.add_system(setup_camera_menu.in_schedule(OnEnter(GameState::Menu)));
        app.add_systems(
            (apply_system_buffers, setup_camera_playing)
                .chain()
                .after(start_game)
                .in_schedule(OnEnter(GameState::Playing)),
        );
        app.add_system(update_cursor_world_pos.in_set(OnUpdate(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct MainCamera;

pub fn spawn_camera(mut commands: Commands, camera_query: Query<&mut Camera, With<MainCamera>>) {
    if camera_query.is_empty() {
        commands
            .spawn(GGFCamera2dBundle::default())
            .insert(Camera2dBundle::default())
            .insert(MainCamera);
    }
}

pub fn setup_camera_menu(
    mut commands: Commands,
    mut camera_query: Query<(&mut Camera, &mut OrthographicProjection), With<MainCamera>>,
) {
    let (mut camera, mut projection) = camera_query.single_mut();
    *projection = OrthographicProjection::default();
}

pub fn setup_camera_playing(
    mut commands: Commands,
    mut camera_query: Query<(&mut Camera, &mut OrthographicProjection), With<MainCamera>>,
    game_data: Res<GameData>,
) {
    let (mut camera, mut projection) = camera_query.single_mut();
    let map_x = (game_data.map_size_x as f32 * (TILE_SIZE + TILE_GAP));
    let map_y = (game_data.map_size_y as f32 * (TILE_SIZE + TILE_GAP));

    projection.scaling_mode = ScalingMode::FixedVertical(map_y + 100.0);
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
