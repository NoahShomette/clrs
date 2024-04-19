use bevy::{
    app::Plugin,
    ecs::{entity::Entity, system::Resource},
    utils::HashMap,
};
use bevy_ggf::object::ObjectId;

pub struct ObjectsPlugin;

impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<ObjectIndex>();
    }
}

#[derive(Resource, Default)]
pub struct ObjectIndex {
    pub hashmap: HashMap<ObjectId, Entity>,
}
