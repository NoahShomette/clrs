use crate::abilities::Abilities;
use crate::actions::{update_actions, Actions};
use crate::color_system::PlayerTileChangedCount;
use crate::loading::AudioAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy_ggf::game_core::Game;
use bevy_ggf::object::ObjectInfo;
use bevy_kira_audio::prelude::*;

pub struct InternalAudioPlugin;

// This plugin is responsible to control the game audio
impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_channel::<EffectSounds>()
            .add_audio_channel::<BackgroundSounds>();

        app.add_plugin(AudioPlugin)
            .add_event::<GameSoundEvents>()
            .add_event::<UiSoundEvents>()
            .add_event::<SoundSettingsEvents>()
            .init_resource::<GameSoundSettings>()
            .add_system(start_audio.in_schedule(OnEnter(GameState::Playing)))
            .add_systems(
                (
                    apply_system_buffers,
                    handle_spawned_object_sounds,
                    control_nuke_sound,
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
struct EffectSounds;
#[derive(Resource)]
struct BackgroundSounds;

#[derive(Resource)]
pub(crate) struct GameSoundSettings {
    is_sound_on: bool,
    is_bg_sound_on: bool,
    sound_level: (f64, f64, f64),
    bg_sound_level: (f64, f64, f64),
    effects_sound_level: (f64, f64, f64),
}

impl Default for GameSoundSettings {
    fn default() -> Self {
        GameSoundSettings {
            is_sound_on: true,
            is_bg_sound_on: true,
            sound_level: (0.0, 0.5, 1.0),
            bg_sound_level: (0.0, 0.15, 1.0),
            effects_sound_level: (0.0, 0.5, 1.0),
        }
    }
}

impl GameSoundSettings {
    fn toggle_sound(&mut self, mut sound_settings_event: &mut EventWriter<SoundSettingsEvents>) {
        self.is_sound_on = !self.is_sound_on;
        sound_settings_event.send(SoundSettingsEvents::SoundToggle(self.is_sound_on));
    }
    fn toggle_bg_sound(&mut self, mut sound_settings_event: &mut EventWriter<SoundSettingsEvents>) {
        self.is_bg_sound_on = !self.is_bg_sound_on;
        sound_settings_event.send(SoundSettingsEvents::BGToggle(self.is_bg_sound_on));
    }
}

pub enum SoundSettingsEvents {
    SoundToggle(bool),
    BGToggle(bool),
    SoundVolumeMaster(f64),
    SoundVolumeBg(f64),
    SoundVolumeEffects(f64),
}

#[derive(PartialEq, Eq)]
pub enum GameSoundEvents {
    Fortify,
    Expand,
    GainTile,
    LostTile,
    Nuke,
    PlaceBuilding,
}

pub enum UiSoundEvents {
    BasicButton,
}

#[derive(Component)]
pub struct ObjectSpawnedSound;

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

fn start_audio(mut commands: Commands, audio: ResMut<bevy_kira_audio::Audio>) {
    commands.insert_resource(FortifyExpandAudio(0));
    commands.insert_resource(GainTileAudio(0));
    commands.insert_resource(LostTileAudio(0));
    commands.insert_resource(MenuAudio(0));
    commands.insert_resource(NukeAudio(0));
    commands.insert_resource(PlaceBuildAudio(0));
}

fn handle_spawned_object_sounds(
    query: Query<(Entity, &ObjectInfo), With<ObjectSpawnedSound>>,
    mut events: EventWriter<GameSoundEvents>,
    mut commands: Commands,
) {
    for (entity, object_info) in query.iter() {
        match object_info.object_type.name.as_str() {
            "Pulser" | "Line" | "Scatter" => events.send(GameSoundEvents::PlaceBuilding),
            "Fortify" => events.send(GameSoundEvents::Fortify),
            "Expand" => events.send(GameSoundEvents::Expand),
            "Nuke" => events.send(GameSoundEvents::Nuke),
            _ => todo!(),
        }
        commands.entity(entity).remove::<ObjectSpawnedSound>();
    }
}

fn control_fortify_expand_sound(
    mut events: EventReader<GameSoundEvents>,
    sound_settings: Res<GameSoundSettings>,
    audio_settings: Res<FortifyExpandAudio>,
    audio_assets: Res<AudioAssets>,
    audio: Res<AudioChannel<EffectSounds>>,
) {
    for sound_event in events.iter() {
        if *sound_event != GameSoundEvents::Fortify && *sound_event != GameSoundEvents::Expand {
            continue;
        }
        audio
            .play(audio_assets.fortify_expand.clone())
            .with_volume(0.3 * sound_settings.effects_sound_level.1);
    }
}

fn control_gain_tile_sound(
    audio_assets: Res<AudioAssets>,
    audio: Res<bevy_kira_audio::Audio>,
    game: Res<Game>,
) {
    let changed_tiles = game.game_world.resource::<PlayerTileChangedCount>();
    if changed_tiles.player_gained_tiles > 0 {
        //audio.play(audio_assets.gain_tile.clone()).with_volume(0.01);
    }
}

fn control_lost_tile_sound(
    audio_assets: Res<AudioAssets>,
    audio: Res<bevy_kira_audio::Audio>,
    game: Res<Game>,
) {
    let changed_tiles = game.game_world.resource::<PlayerTileChangedCount>();
    if changed_tiles.player_lost_tiles > 0 {
        //audio.play(audio_assets.lost_tile.clone()).with_volume(0.01);
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
    mut events: EventReader<GameSoundEvents>,
    sound_settings: Res<GameSoundSettings>,
    audio_settings: Res<NukeAudio>,
    audio_assets: Res<AudioAssets>,
    audio: Res<AudioChannel<EffectSounds>>,
) {
    for sound_event in events.iter() {
        if *sound_event != GameSoundEvents::Nuke {
            continue;
        }
        audio
            .play(audio_assets.nuke.clone())
            .with_volume(0.3 * sound_settings.effects_sound_level.1);
    }
}

fn control_place_build_sound(
    mut events: EventReader<GameSoundEvents>,
    sound_settings: Res<GameSoundSettings>,
    audio_settings: Res<PlaceBuildAudio>,
    audio_assets: Res<AudioAssets>,
    audio: Res<AudioChannel<EffectSounds>>,
) {
    for sound_event in events.iter() {
        if *sound_event != GameSoundEvents::PlaceBuilding {
            continue;
        }
        audio
            .play(audio_assets.place_build.clone())
            .with_volume(0.3 * sound_settings.effects_sound_level.1);
    }
}
