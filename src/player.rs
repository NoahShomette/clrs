use crate::actions::Actions;
use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use ns_defaults::camera::CursorWorldPos;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_player.in_set(OnUpdate(GameState::Playing)));

        app.init_resource::<CursorWorldPos>()
            .add_system(update_cursor_world_pos);
    }
}

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
        cursor_world_pos.cursor_world_pos = current_cursor_position;
    }
}

fn move_player(
    time: Res<Time>,
    actions: Res<Actions>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    if actions.player_movement.is_none() {
        return;
    }
    let speed = 150.;
    let movement = Vec3::new(
        actions.player_movement.unwrap().x * speed * time.delta_seconds(),
        actions.player_movement.unwrap().y * speed * time.delta_seconds(),
        0.,
    );
    for mut player_transform in &mut player_query {
        player_transform.translation += movement;
    }
}
