use bevy::app::App;
use bevy::prelude::Plugin;

pub mod map;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {}
}
