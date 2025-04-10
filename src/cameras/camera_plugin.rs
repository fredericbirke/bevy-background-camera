use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::render::view::RenderLayers;
use bevy::window::WindowResized;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, update_camera_zoom); // Keep zoom update // Keep zoom update
    }
}

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: CameraLayers::Game as isize,
            clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            hdr: false,
            ..default()
        },
        RenderLayers::from_layers(&[CameraLayers::Game as usize])
            .without(CameraLayers::Background as usize),
    ));
    for num in 0..20 {
        commands.spawn((
            Sprite::from_image(asset_server.load("grid-outline.png")),
            Transform::from_xyz(-1000. + 100. * num as f32, 0., 0.),
            RenderLayers::layer(CameraLayers::Game as usize),
        ));
    }
}

pub enum CameraLayers {
    Background = 0,
    Game = 1,
}
fn update_camera_zoom(
    mut resize_events: EventReader<WindowResized>,
    mut query: Query<&mut OrthographicProjection>,
) {
    // Reference resolution
    const REFERENCE_WIDTH: f32 = 2560.0; // Width of the reference resolution
    const REFERENCE_HEIGHT: f32 = 1440.0; // Height of the reference resolution

    for event in resize_events.read() {
        for mut projection in query.iter_mut() {
            // Calculate zoom factor based on the reference resolution
            let zoom_factor_x = event.width / REFERENCE_WIDTH;
            let zoom_factor_y = event.height / REFERENCE_HEIGHT;

            // Use the average zoom factor or adjust based on your needs
            let zoom_factor = (zoom_factor_x + zoom_factor_y) / 2.0;

            // Adjust the scaling mode to reflect the new zoom level
            projection.scaling_mode = ScalingMode::WindowSize;
            projection.scale = 1.0 / zoom_factor;
        }
    }
}
