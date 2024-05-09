// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use bevy::DefaultPlugins;
use bevy_splash_screen::{SplashAssetType, SplashItem, SplashPlugin, SplashScreen};
use bevy_tweening::{EaseFunction, TweeningPlugin};
use bevy_vector_shapes::Shape2dPlugin;
use clrs::{GamePlugin, GameState};
use std::io::Cursor;
use std::time::Duration;
use winit::window::Icon;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "clrs".to_string(),
                        fit_canvas_to_parent: true,
                        canvas: Some("#bevy".to_owned()),
                        #[cfg(not(target_arch = "wasm32"))]
                        resolution: (1280., 720.).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        //.add_plugin(ns_defaults::camera::CameraPlugin)
        .add_plugin(TweeningPlugin)
        .add_plugin(Shape2dPlugin::default())
        .add_plugin(GamePlugin)
        .add_system(set_window_icon.on_startup())
        /*
        .add_plugin(
            SplashPlugin::new(GameState::Splash, GameState::Menu)
                .add_screen(SplashScreen {
                    brands: vec![SplashItem {
                        asset: SplashAssetType::SingleText(
                            Text::from_sections([
                                TextSection::new(
                                    "CLRS\n",
                                    TextStyle {
                                        font_size: 40.,
                                        color: Color::WHITE,
                                        ..default()
                                    },
                                ),
                                TextSection::new(
                                    "by\n",
                                    TextStyle {
                                        font_size: 24.,
                                        color: Color::WHITE.with_a(0.75),
                                        ..default()
                                    },
                                ),
                                TextSection::new(
                                    "Noah & Kolbe",
                                    TextStyle {
                                        font_size: 32.,
                                        color: Color::WHITE,
                                        ..default()
                                    },
                                ),
                            ])
                            .with_alignment(TextAlignment::Center),
                            "fonts/FiraSans-Bold.ttf".to_string(),
                        ),
                        tint: Color::WHITE,
                        size: Size::new(Val::Auto, Val::Px(150.)),
                        ease_function: EaseFunction::QuarticInOut.into(),
                        duration: Duration::from_secs_f32(1.),
                        is_static: false,
                    }],
                    background_color: BackgroundColor(Color::BLACK),
                    ..default()
                })
                .add_screen(SplashScreen {
                    brands: vec![
                        SplashItem {
                            asset: SplashAssetType::SingleText(
                                Text::from_sections([
                                    TextSection::new(
                                        "Made\n",
                                        TextStyle {
                                            font_size: 40.,
                                            color: Color::WHITE,
                                            ..default()
                                        },
                                    ),
                                    TextSection::new(
                                        "In\n",
                                        TextStyle {
                                            font_size: 24.,
                                            color: Color::WHITE.with_a(0.75),
                                            ..default()
                                        },
                                    ),
                                    TextSection::new(
                                        "Bevy",
                                        TextStyle {
                                            font_size: 32.,
                                            color: Color::WHITE,
                                            ..default()
                                        },
                                    ),
                                ])
                                .with_alignment(TextAlignment::Center),
                                "fonts/FiraSans-Bold.ttf".to_string(),
                            ),
                            tint: Color::WHITE,
                            size: Size::new(Val::Percent(30.), Val::Px(150.)),
                            ease_function: EaseFunction::QuarticInOut.into(),
                            duration: Duration::from_secs_f32(1.),
                            is_static: false,
                        },
                        SplashItem {
                            asset: SplashAssetType::SingleImage("bevy.png".to_string()),
                            tint: Color::WHITE,
                            size: Size::new(Val::Auto, Val::Percent(40.0)),
                            ease_function: EaseFunction::QuinticInOut.into(),
                            duration: Duration::from_secs_f32(1.),
                            is_static: true,
                        },
                    ],
                    background_color: BackgroundColor(Color::BLACK),
                    ..default()
                }),
        )*/
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
