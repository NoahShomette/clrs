mod abilities;
mod actions;
mod ai;
mod audio;
mod buildings;
mod camera;
mod color_system;
mod draw;
mod framework;
mod game;
mod loading;
mod mapping;
mod objects;
mod pathfinding;
mod player;
mod ui;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::loading::{level_loader, LoadingPlugin};
use crate::player::PlayerPlugin;

use crate::camera::CameraPlugin;
use crate::color_system::ColorSystemPlugin;
use crate::draw::DrawPlugin;
use crate::framework::FrameworkPlugin;
use crate::game::GameCorePlugin;
use crate::loading::colors_loader::PalettesAssets;
use crate::mapping::MapPlugin;
use crate::ui::UiPlugin;
use bevy::app::App;

#[cfg(not(debug_assertions))]
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ggf::BggfDefaultPlugins;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use objects::ObjectsPlugin;

/// Generic Game State
/// Update with a separate Pause state eventually
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // Splashscreen state
    Splash,
    // During this State the actual game logic is executed
    Playing,
    /// Game has ended and will show a game over screen and then go to main menu
    Ended,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GamePausedState {
    // The game is not paused
    #[default]
    NotPaused,
    /// Game is paused and will return to playing, quit, or menu
    Paused,
}

/// testing
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set(base)]
pub enum MenuSystemSet {
    Pre,
    PreCommandFlush,
    Main,
    PostCommandFlush,
    Post,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // crate plugins
        app.add_plugins(BggfDefaultPlugins);
        app.add_plugin(RonAssetPlugin::<level_loader::Levels>::new(&["levels.ron"]));
        app.add_plugin(RonAssetPlugin::<PalettesAssets>::new(&["palettes.ron"]));

        app.add_state::<GameState>()
            .add_state::<GamePausedState>()
            .add_plugin(ColorSystemPlugin)
            .add_plugin(LoadingPlugin)
            .add_plugin(UiPlugin)
            .add_plugin(DrawPlugin)
            .add_plugin(ActionsPlugin)
            .add_plugin(InternalAudioPlugin)
            .add_plugin(CameraPlugin)
            .add_plugin(PlayerPlugin)
            .add_plugin(GameCorePlugin)
            .add_plugin(FrameworkPlugin)
            .add_plugin(MapPlugin)
            .add_plugin(ObjectsPlugin);
        #[cfg(not(debug_assertions))]
        {
            app.add_plugin(FrameTimeDiagnosticsPlugin::default());
        }

        #[cfg(debug_assertions)]
        {
            app.add_plugin(FrameTimeDiagnosticsPlugin::default())
                .add_plugin(LogDiagnosticsPlugin::default())
                .add_plugin(WorldInspectorPlugin::new());
        }
    }
}
