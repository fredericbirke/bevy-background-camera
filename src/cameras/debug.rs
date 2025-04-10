use bevy::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_debug_objects)
            .add_systems(Update, log_frame_count);
    }
}

// Frame counter for debugging
#[derive(Resource, Default)]
pub struct FrameCounter {
    pub count: u32,
}

// System to log every few frames to avoid console spam
fn log_frame_count(mut counter: Local<FrameCounter>) {
    counter.count += 1;
    if counter.count % 60 == 0 {
        info!("Rendered frame {}", counter.count);
    }
}

// Spawn debug objects for the background camera to render
fn setup_debug_objects(_commands: Commands, _asset_server: Res<AssetServer>) {
    info!("Setting up debug objects for background camera");
    // Background layer objects (RED layer)
    // commands.spawn((
    //     Sprite {
    //         color: Color::srgb(1.0, 0.0, 0.0), // RED
    //         custom_size: Some(Vec2::new(500.0, 500.0)),
    //         ..default()
    //     },
    //     Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    //     RenderLayers::layer((CameraLayers::Background as u8).into()),
    // ));

    // // Add more background shapes at different positions
    // for i in 0..5 {
    //     let x = (i as f32 - 2.0) * 200.0;
    //     commands.spawn((
    //         Sprite {
    //             color: Color::srgb(1.0, 0.5, 0.5), // Light red
    //             custom_size: Some(Vec2::new(100.0, 100.0)),
    //             ..default()
    //         },
    //         Transform::from_translation(Vec3::new(x, 200.0, 0.0)),
    //         RenderLayers::layer((CameraLayers::Background as u8).into()),
    //     ));
    // }

    // commands.spawn((
    //     Sprite {
    //         image: asset_server.load("levelData/background/Background-Image-try_.png"),
    //         ..Default::default()
    //     },
    //     Transform {
    //         translation: Vec3::new(0.0, 0.0, -900.0),
    //         scale: Vec3::new(0.5, 0.5, 1.0),
    //         ..Default::default()
    //     },
    //     RenderLayers::layer((CameraLayers::Game as u8).into()),
    // ));
    // Main camera objects (GREEN layer)
    // commands.spawn((
    //     Sprite {
    //         color: Color::srgb(0.0, 1.0, 0.0), // GREEN
    //         custom_size: Some(Vec2::new(300.0, 300.0)),
    //         ..default()
    //     },
    //     Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
    //     RenderLayers::layer((CameraLayers::Game as u8).into()),
    // ));

    // // Add more game layer shapes
    // for i in 0..3 {
    //     let x = (i as f32 - 1.0) * 150.0;
    //     commands.spawn((
    //         Sprite {
    //             color: Color::srgb(0.0, 0.0, 1.0), // BLUE
    //             custom_size: Some(Vec2::new(80.0, 80.0)),
    //             ..default()
    //         },
    //         Transform::from_translation(Vec3::new(x, -150.0, 0.1)),
    //         RenderLayers::layer((CameraLayers::Game as u8).into()),
    //     ));
    // }
}
