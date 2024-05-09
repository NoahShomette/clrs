use bevy::{
    app::Plugin,
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        query::{Changed, With},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::BuildChildren,
    input::{keyboard::KeyCode, Input},
    prelude::default,
    render::color::Color,
    text::{Text, TextAlignment, TextSection, TextStyle},
    ui::{
        node_bundles::TextBundle, widget::Button, AlignItems, BackgroundColor, Interaction,
        JustifyContent, PositionType, Size, Style, UiRect, Val,
    },
};
use bevy_ggf::game_core::Game;

use crate::{
    audio::{GameSoundSettings, SoundSettingsEvents},
    loading::FontAssets,
};

use super::{menu::back_and_forth_button, modal_panel, DisabledButton, ModalStyle, PlayerColors};

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems((handle_buttons, update_sound_level));
    }
}

#[derive(Component)]
pub struct DecreaseSoundButton;

#[derive(Component)]
pub struct IncreaseSoundButton;

#[derive(Component)]
pub struct SoundText;

pub fn spawn_settings_menu<MenuMarker: Component + Clone, CloseButtonBundle: Bundle>(
    menu_marker: MenuMarker,
    modal_style: ModalStyle<CloseButtonBundle>,
    mut commands: &mut Commands,
    font_assets: &Res<FontAssets>,
    player_colors: &PlayerColors,
    sound_settings: &GameSoundSettings,
) {
    let modal_content = modal_panel(
        menu_marker.clone(),
        modal_style,
        &mut commands,
        &font_assets,
    );

    commands.entity(modal_content).with_children(|parent| {
        let backward = sound_settings.sound_level() > sound_settings.sound_min();
        let forward = sound_settings.sound_level() < sound_settings.sound_max();

        back_and_forth_button(
            parent,
            &font_assets,
            menu_marker,
            DecreaseSoundButton,
            backward,
            IncreaseSoundButton,
            forward,
            "Sound",
        );

        parent
            .spawn(
                TextBundle::from_section(
                    format!("{:.2}", sound_settings.sound_level(),),
                    TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 40.0,
                        color: Color::GRAY,
                    },
                )
                .with_text_alignment(TextAlignment::Center)
                .with_style(Style {
                    position_type: PositionType::Relative,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(5.0)),
                    size: Size::new(Val::Auto, Val::Auto),
                    ..default()
                }),
            )
            .insert(SoundText);
    });
}

fn handle_buttons(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            Option<&DisabledButton>,
            Option<&DecreaseSoundButton>,
            Option<&IncreaseSoundButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut sound_settings: ResMut<GameSoundSettings>,
) {
    for (_, interaction, option_disabled, opt_dsb, opt_isb) in &mut interaction_query {
        if Interaction::Clicked != *interaction {
            continue;
        }

        if option_disabled.is_some() {
            continue;
        }

        let modifier = keyboard_input.pressed(KeyCode::LShift);

        if opt_dsb.is_some() {
            sound_settings.decrease_sound_level();
        }

        if opt_isb.is_some() {
            sound_settings.increase_sound_level();
        }
    }
}

fn update_sound_level(
    mut colors: Query<(&SoundText, &mut Text)>,
    mut buttons: Query<(
        Entity,
        Option<&DisabledButton>,
        Option<&DecreaseSoundButton>,
        Option<&IncreaseSoundButton>,
        &mut BackgroundColor,
    )>,
    game_sound_settings: Res<GameSoundSettings>,
    mut commands: Commands,
) {
    for (_, mut text) in colors.iter_mut() {
        text.sections[0].value = format!("{:.2}", game_sound_settings.sound_level());
    }

    for (entity, option_disabled_button, option_1, option_2, mut background_color) in
        buttons.iter_mut()
    {
        if game_sound_settings.sound_level() == game_sound_settings.sound_min() {
            if let Some(_) = option_1 {
                if let None = option_disabled_button {
                    background_color.0 = Color::DARK_GRAY;
                    commands.entity(entity).insert(DisabledButton);
                }
            }
            if let Some(_) = option_2 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
        }
        if game_sound_settings.sound_level() == game_sound_settings.sound_max() {
            if let Some(_) = option_1 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
            if let Some(_) = option_2 {
                if let None = option_disabled_button {
                    background_color.0 = Color::DARK_GRAY;
                    commands.entity(entity).insert(DisabledButton);
                }
            }
        }

        if game_sound_settings.sound_level() > game_sound_settings.sound_min()
            && game_sound_settings.sound_level() < game_sound_settings.sound_max()
        {
            if let Some(_) = option_1 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
            if let Some(_) = option_2 {
                if let Some(_) = option_disabled_button {
                    background_color.0 = Color::GRAY;
                    commands.entity(entity).remove::<DisabledButton>();
                }
            }
        }
    }
}
