pub mod colors_loader;
pub mod level_loader;

use crate::loading::colors_loader::PalettesHandle;
use crate::loading::level_loader::LevelHandle;
use crate::ui::PlayerColors;
use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Menu),
        )
        .add_collection_to_loading_state::<_, FontAssets>(GameState::Loading)
        .add_collection_to_loading_state::<_, AudioAssets>(GameState::Loading)
        .add_collection_to_loading_state::<_, LevelHandle>(GameState::Loading)
        .add_collection_to_loading_state::<_, TextureAssets>(GameState::Loading)
        .add_collection_to_loading_state::<_, PalettesHandle>(GameState::Loading);

        app.init_resource_after_loading_state::<_, PlayerColors>(GameState::Loading);
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/nuke.wav")]
    pub nuke: Handle<AudioSource>,
    #[asset(path = "audio/fortify_expand.wav")]
    pub fortify_expand: Handle<AudioSource>,
    //#[asset(path = "audio/gain_tile.wav")]
    // pub gain_tile: Handle<AudioSource>,
    #[asset(path = "audio/lost_tile.wav")]
    pub lost_tile: Handle<AudioSource>,
    #[asset(path = "audio/menu.wav")]
    pub menu: Handle<AudioSource>,
    #[asset(path = "audio/place_build.wav")]
    pub place_build: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/pulser.png")]
    pub pulser: Handle<Image>,
    #[asset(path = "textures/scatter.png")]
    pub scatter: Handle<Image>,
    #[asset(path = "textures/line.png")]
    pub line: Handle<Image>,
    #[asset(path = "textures/nuke.png")]
    pub nuke: Handle<Image>,
    #[asset(path = "textures/fortify.png")]
    pub fortify: Handle<Image>,
    #[asset(path = "textures/expand.png")]
    pub expand: Handle<Image>,
}
