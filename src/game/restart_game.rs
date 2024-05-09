use bevy::{
    app::{IntoSystemAppConfig, Plugin},
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        query::With,
        schedule::{NextState, OnEnter},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    hierarchy::DespawnRecursiveExt,
    prelude::default,
    render::color::Color,
    ui::{
        node_bundles::NodeBundle, AlignItems, FlexDirection, JustifyContent, PositionType, Size,
        Style, UiRect, Val, ZIndex,
    },
};

use crate::GameState;

pub struct RestartGamePlugin;

impl Plugin for RestartGamePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<RestartGameEvent>();
        app.add_system(handle_restart_game_events);
        app.add_system(
            crate::game::restart_game::remove_restart_game_screen
                .in_schedule(OnEnter(GameState::Playing)),
        )
        .add_system(
            crate::game::restart_game::restart_game_in_menu.in_schedule(OnEnter(GameState::Menu)),
        );
    }
}

/// If this resource is present the main menu will automatically restart the game
#[derive(Resource, Default)]
pub struct RestartGame;

#[derive(Component, Default)]
pub struct RestartGameBlocker;

pub struct RestartGameEvent;

/// Fn that actually restarts the game in the main menu
pub fn handle_restart_game_events(
    mut restart_game: EventReader<RestartGameEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    for _ in restart_game.iter() {
        next_state.set(GameState::Menu);
        commands.init_resource::<RestartGame>();
        commands.spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Relative,
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                z_index: ZIndex::Global(1000),
                background_color: Color::rgba(0.0, 0.0, 0.0, 1.0).into(),
                ..default()
            },
            RestartGameBlocker,
        ));
    }
}

/// Fn that actually restarts the game in the main menu
pub fn restart_game_in_menu(
    restart_game: Option<Res<RestartGame>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    if restart_game.is_some() {
        next_state.set(GameState::Playing);
        commands.remove_resource::<RestartGame>();
    }
}

/// Remvoes the blocker screen when we enter the game mode
pub fn remove_restart_game_screen(
    query: Query<Entity, With<RestartGameBlocker>>,
    mut commands: Commands,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
