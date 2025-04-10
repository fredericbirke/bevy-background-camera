use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::view::RenderLayers;
use bevy::window::WindowResized;

use crate::cameras::camera_plugin::CameraLayers;

// Marker component for the background camera
#[derive(Component)]
pub struct BackgroundCamera;

// Component to hold the handle for the background LUT
#[derive(Component, Clone, ExtractComponent, Default)] // Make sure ExtractComponent is derived
pub struct BackgroundLutSource {
    pub lut_texture: Handle<Image>,
}

// Resource to hold the handle to the offscreen render target image
#[derive(Resource, Clone, ExtractResource)]
pub struct BackgroundRenderTarget {
    pub handle: Handle<Image>,
}
#[derive(Resource, Clone, ExtractResource)]
pub struct BackgroundProcessedRenderTarget {
    pub handle: Handle<Image>,
}

// Default implementations to support init_resource
impl Default for BackgroundRenderTarget {
    fn default() -> Self {
        Self {
            handle: Handle::default(),
        }
    }
}

impl Default for BackgroundProcessedRenderTarget {
    fn default() -> Self {
        Self {
            handle: Handle::default(),
        }
    }
}
const BACKGROUND_LUT_PATH: &str = "shaders/background_lut.png"; // <-- Your specific background LUT

pub struct BackgroundCameraPlugin;

impl Plugin for BackgroundCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<BackgroundLutSource>::default()) // Extract the LUT source
            .add_plugins(ExtractResourcePlugin::<BackgroundRenderTarget>::default())
            .add_plugins(ExtractResourcePlugin::<BackgroundProcessedRenderTarget>::default())
            .init_resource::<BackgroundRenderTarget>()
            .init_resource::<BackgroundProcessedRenderTarget>()
            .add_systems(Startup, setup_background_camera)
            .add_systems(Update, resize_background_render_target);
    }
}

fn setup_background_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
) {
    info!("Setting up background camera");
    let window = windows.single();
    let size = Extent3d {
        width: window.resolution.physical_width(),
        height: window.resolution.physical_height(),
        ..default()
    };

    // Create the image asset for the render target
    let render_target_image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("background_render_target"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(), // Use the default format
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT, // Important!
            view_formats: &[],
        },
        ..default()
    };
    // Fill with a transparent color initially
    let mut render_target_image_clone = render_target_image.clone();
    render_target_image_clone.resize(size);
    // Initialize with transparent pixels
    render_target_image_clone.data.fill(0);

    let render_target_handle = images.add(render_target_image_clone.clone());

    // Store the handle in a resource
    commands.insert_resource(BackgroundRenderTarget {
        handle: render_target_handle.clone(),
    });
    let mut processed_target_image = render_target_image_clone;
    processed_target_image.texture_descriptor.label = Some("background_processed_render_target");
    let processed_target_handle = images.add(processed_target_image);
    commands.insert_resource(BackgroundProcessedRenderTarget {
        handle: processed_target_handle.clone(),
    });

    // Debug log the size of render targets and handles
    info!(
        "Created background render target: {}x{}",
        size.width, size.height
    );
    info!(
        "Background render target handle: {:?}",
        render_target_handle
    );
    info!(
        "Background processed render target handle: {:?}",
        processed_target_handle
    );
    // Load the background LUT
    let background_lut_handle: Handle<Image> = asset_server.load(BACKGROUND_LUT_PATH);

    // Spawn the background camera
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: CameraLayers::Background as isize, // Render first
            target: bevy::render::camera::RenderTarget::Image(render_target_handle.clone()), // Render to our image!
            clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 0.0)), // Set to transparent background
            ..default()
        },
        RenderLayers::from_layers(&[CameraLayers::Background as usize]),
        BackgroundCamera, // Marker component
        BackgroundLutSource {
            lut_texture: background_lut_handle,
        },
    ));
    commands.spawn((
        Sprite {
            image: asset_server.load("forrest_wqhd.png"),
            custom_size: Some(Vec2::new(2560.0, 1440.0)),
            ..Default::default()
        },
        Transform {
            ..Default::default()
        },
        RenderLayers::layer((CameraLayers::Background as u8).into()),
    ));
}

// System to resize the render target when the window resize
fn resize_background_render_target(
    mut resize_events: EventReader<WindowResized>,
    mut images: ResMut<Assets<Image>>,
    background_target: Res<BackgroundRenderTarget>,
    background_processed_target: Res<BackgroundProcessedRenderTarget>,
) {
    for event in resize_events.read() {
        if let Some(image) = images.get_mut(&background_target.handle) {
            let size = Extent3d {
                width: event.width as u32,   // Use event physical size
                height: event.height as u32, // Use event physical size
                ..default()
            };
            image.resize(size);
        }
        if let Some(image) = images.get_mut(&background_processed_target.handle) {
            let size = Extent3d {
                width: event.width as u32,
                height: event.height as u32,
                ..default()
            };
            image.resize(size);
        }
    }
}
