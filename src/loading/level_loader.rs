use crate::abilities::Abilities;
use crate::buildings::BuildingTypes;
use bevy::prelude::*;
use bevy::prelude::{Handle, Resource};
use bevy::reflect::TypeUuid;
use bevy_asset_loader::prelude::AssetCollection;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, TypeUuid)]
#[uuid = "d9aec76d-0e89-4fb7-bf19-54c051fae561"]
pub struct LevelHandle {
    #[asset(path = "defaults.levels.ron")]
    pub levels: Handle<Levels>,
}

#[derive(serde::Deserialize, TypeUuid)]
#[uuid = "d9aec76d-0e89-4fb7-bf19-54c051fae268"]
pub struct Levels {
    pub levels: Vec<Level>,
}

#[derive(serde::Deserialize, Reflect, FromReflect, Clone)]
pub struct Level {
    pub name: String,
    pub spawn_points: Vec<(usize, usize)>,
    pub tiles: Vec<Vec<TileType>>,
}

#[derive(serde::Deserialize, Reflect, FromReflect, Clone)]
pub enum TileType {
    Colorable,
    NonColorable,
}
