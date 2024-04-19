use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Resource;
use bevy::prelude::Plugin;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::tiles::TilePos;

pub mod map;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapTileIndex>();
    }
}

#[derive(Resource, Default)]
pub struct MapTileIndex {
    pub hashmap: HashMap<TilePos, Entity>,
}
