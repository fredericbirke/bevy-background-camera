mod cameras;

use bevy::{
    prelude::*,
    window::{PresentMode, WindowMode, WindowResolution, WindowTheme},
};
use cameras::{
    background_camera::BackgroundCameraPlugin, background_lut::BackgroundLutPlugin,
    camera_plugin::CameraPlugin, composite_pass::CompositePlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set({
            WindowPlugin {
                primary_window: Some(Window {
                    title: "I am a window!".into(),
                    name: Some("bevy.app".into()),
                    present_mode: PresentMode::AutoVsync,
                    mode: WindowMode::Windowed,
                    resolution: WindowResolution::new(1280., 800.),
                    window_theme: Some(WindowTheme::Dark),
                    visible: true,
                    ..default()
                }),
                ..default()
            }
        }))
        .add_plugins((
            BackgroundLutPlugin,
            CameraPlugin,
            CompositePlugin,
            BackgroundCameraPlugin,
        ))
        .run();
}
