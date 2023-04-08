use crate::abilities::Abilities;
use crate::actions::{update_actions, Actions};
use crate::color_system::PlayerTileChangedCount;
use crate::loading::AudioAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy_ggf::game_core::Game;
use bevy_kira_audio::prelude::*;

pub struct InternalAudioPlugin;

// This plugin is responsible to control the game audio
impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .add_system(start_audio.in_schedule(OnEnter(GameState::Playing)))
            .add_systems(
                (
                    control_nuke_sound.before(update_actions),
                    control_fortify_expand_sound,
                    control_gain_tile_sound,
                    control_lost_tile_sound,
                    control_menu_sound,
                    control_place_build_sound,
                )
                    .chain()
                    .in_set(OnUpdate(GameState::Playing)),
            );
    }
}

#[derive(Resource)]
struct FortifyExpandAudio(u32);
#[derive(Resource)]
struct GainTileAudio(u32);
#[derive(Resource)]
struct LostTileAudio(u32);
#[derive(Resource)]
struct MenuAudio(u32);
#[derive(Resource)]
struct NukeAudio(u32);
#[derive(Resource)]
struct PlaceBuildAudio(u32);

fn start_audio(mut commands: Commands) {
    commands.insert_resource(FortifyExpandAudio(0));
    commands.insert_resource(GainTileAudio(0));
    commands.insert_resource(LostTileAudio(0));
    commands.insert_resource(MenuAudio(0));
    commands.insert_resource(NukeAudio(0));
    commands.insert_resource(PlaceBuildAudio(0));
}

fn control_fortify_expand_sound(
    actions: Query<&Actions>,
    audio_settings: Res<FortifyExpandAudio>,
    audio_assets: Res<AudioAssets>,
    audio: Res<bevy_kira_audio::Audio>,
) {
    for action in actions.iter() {
        if (action.selected_ability == Abilities::Fortify
            || action.selected_ability == Abilities::Expand)
            && action.placed_ability
            && audio_settings.0 < 3
        {
            audio
                .play(audio_assets.fortify_expand.clone())
                .with_volume(0.2);
        }
    }
}

fn control_gain_tile_sound(
    audio_assets: Res<AudioAssets>,
    audio: Res<bevy_kira_audio::Audio>,
    game: Res<Game>,
) {
    let changed_tiles = game.game_world.resource::<PlayerTileChangedCount>();
    if changed_tiles.player_gained_tiles > 0 {
        audio.play(audio_assets.gain_tile.clone()).with_volume(0.1);
    }
}

fn control_lost_tile_sound(
    audio_assets: Res<AudioAssets>,
    audio: Res<bevy_kira_audio::Audio>,
    game: Res<Game>,
) {
    let changed_tiles = game.game_world.resource::<PlayerTileChangedCount>();
    if changed_tiles.player_lost_tiles > 0 {
        audio.play(audio_assets.lost_tile.clone()).with_volume(0.01);
    }
}

fn control_menu_sound(
    actions: Query<&Actions>,
    audio_settings: Res<MenuAudio>,
    audio_assets: Res<AudioAssets>,
    audio: Res<bevy_kira_audio::Audio>,
) {
    for action in actions.iter() {
        if action.selected_ability == Abilities::Nuke
            && action.placed_ability
            && audio_settings.0 < 3
        {
            //audio.play(audio_assets.menu.clone()).with_volume(0.3);
        }
    }
}

fn control_nuke_sound(
    actions: Query<&Actions>,
    audio_settings: Res<NukeAudio>,
    audio_assets: Res<AudioAssets>,
    audio: Res<bevy_kira_audio::Audio>,
) {
    for action in actions.iter() {
        if action.selected_ability == Abilities::Nuke
            && action.placed_ability
            && audio_settings.0 < 3
        {
            audio.play(audio_assets.nuke.clone()).with_volume(0.2);
        }
    }
}

fn control_place_build_sound(
    actions: Query<&Actions>,
    audio_settings: Res<PlaceBuildAudio>,
    audio_assets: Res<AudioAssets>,
    audio: Res<bevy_kira_audio::Audio>,
) {
    for action in actions.iter() {
        if action.placed_building && audio_settings.0 < 3 {
            audio
                .play(audio_assets.place_build.clone())
                .with_volume(0.2);
        }
    }
}
