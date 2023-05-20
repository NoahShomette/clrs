// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowMode};
use bevy::winit::WinitWindows;
use bevy::DefaultPlugins;
use clrs::GamePlugin;
use std::io::Cursor;
use bevy_tweening::TweeningPlugin;
use bevy_vector_shapes::Shape2dPlugin;
use winit::window::Icon;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "clrs".to_string(),
                fit_canvas_to_parent: true,
                canvas: Some("#bevy".to_owned()),
                #[cfg(not(target_arch = "wasm32"))]
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(ns_defaults::camera::CameraPlugin)
        .add_plugin(TweeningPlugin)
        .add_plugin(Shape2dPlugin::default())
        .add_plugin(GamePlugin)
        .add_system(set_window_icon.on_startup())
        .run();
}

// Sets the icon on windows and X11
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_entity = primary_window.single();
    let primary = windows.get_window(primary_entity).unwrap();
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}
