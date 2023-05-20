use crate::ui::Palette;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_asset_loader::prelude::AssetCollection;

#[derive(AssetCollection, Resource, TypeUuid)]
#[uuid = "d122a2e7-3d17-4ab9-9c20-96767c6d6e44"]
pub struct PalettesHandle {
    #[asset(path = "defaults.palettes.ron")]
    pub palettes: Handle<PalettesAssets>,
}

#[derive(serde::Deserialize, TypeUuid)]
#[uuid = "48584d80-0365-4f24-99a3-6afd4e5b275a"]
pub struct PalettesAssets {
    pub palettes: Vec<Palette>,
}
