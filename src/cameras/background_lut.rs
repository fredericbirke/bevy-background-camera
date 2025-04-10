// Nightdrawn-Tower-Defense/client/src/cameras/background_camera.rs
// ... (add this plugin struct and implementation)

use bevy::{
    app::{App, Plugin},
    asset::DirectAssetAccessExt,
    core_pipeline::{
        core_2d::graph::Core2d, fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    ecs::{
        query::QueryItem,
        system::Resource,
        world::{FromWorld, World},
    },
    image::BevyDefault,
    log::{info, warn},
    render::{
        RenderApp,
        camera::{ExtractedCamera, NormalizedRenderTarget},
        render_asset::RenderAssets,
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            AddressMode, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, FilterMode, FragmentState,
            MultisampleState, Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, TextureFormat, TextureSampleType,
            binding_types::{sampler, texture_2d},
        },
        renderer::{RenderContext, RenderDevice},
        texture::GpuImage,
    },
};

use super::background_camera::{
    BackgroundLutSource, BackgroundProcessedRenderTarget, BackgroundRenderTarget,
};

const SHADER_ASSET_PATH: &str = "shaders/night_shader.wgsl";
// --- Background LUT Post Processing ---

pub struct BackgroundLutPlugin;

impl Plugin for BackgroundLutPlugin {
    fn build(&self, app: &mut App) {
        // BackgroundCameraPlugin already adds these plugins, don't add them again

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_render_graph_node::<ViewNodeRunner<BackgroundLutNode>>(Core2d, BackgroundLutLabel);

        // Add the node to the render graph
        // Run this node specifically after the background camera finishes its main pass
        // and before the main camera's Tonemapping node, ensuring the processed background
        // texture is ready for the Composite Pass.
        // Note: The ViewNodeRunner implicitly handles running only for the correct camera target.
        // Render graph edges are now defined in CompositePlugin.
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<BackgroundLutPipeline>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct BackgroundLutLabel;

#[derive(Default)]
struct BackgroundLutNode;

impl ViewNode for BackgroundLutNode {
    // Query for background cameras specifically
    type ViewQuery = (
        &'static ExtractedCamera,
        &'static BackgroundLutSource,
        // &'static ViewTarget, // We get the target from ExtractedCamera
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (extracted_camera, lut_source): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        info!("Running BackgroundLutNode");

        // Check if resource exists before trying to access it
        if !world.contains_resource::<BackgroundRenderTarget>() {
            warn!(
                "BackgroundRenderTarget is missing in render world, skipping background LUT pass"
            );
            return Ok(());
        }

        if !world.contains_resource::<BackgroundProcessedRenderTarget>() {
            warn!(
                "BackgroundProcessedRenderTarget is missing in render world, skipping background LUT pass"
            );
            return Ok(());
        }

        // Now we can safely get the resource
        let source_target = world.resource::<BackgroundRenderTarget>();

        // Get the ID of the target image resource we expect the background camera to render to.
        let expected_target_id = source_target.handle.id();

        // Check if the camera currently being processed by this node
        // is targeting an image AND if that image's handle ID matches our expected background target ID.
        let is_correct_target = match &extracted_camera.target {
            Some(NormalizedRenderTarget::Image(handle)) => handle.id() == expected_target_id,
            _ => false, // Not rendering to an image or not the right one
        };

        if !is_correct_target {
            // This node instance is not running for the correct background camera, skip.
            return Ok(());
        }
        // --- End: Correct Check ---

        // Now we know extracted_camera.target is Some(NormalizedRenderTarget::Image(handle))
        // and the handle ID matches. We can safely proceed.

        let pipeline_cache = world.resource::<PipelineCache>();
        let background_lut_pipeline = world.resource::<BackgroundLutPipeline>();

        // Get the handles for source and destination
        let source_target = world.resource::<BackgroundRenderTarget>();
        let destination_target = world.resource::<BackgroundProcessedRenderTarget>();

        let Some(pipeline) =
            pipeline_cache.get_render_pipeline(background_lut_pipeline.pipeline_id)
        else {
            // Pipeline not ready
            return Ok(());
        };
        let gpu_images = world.resource::<RenderAssets<GpuImage>>();
        let Some(lut_gpu_image) = gpu_images.get(&lut_source.lut_texture) else {
            // LUT texture not ready on GPU
            return Ok(());
        };

        // Get the GpuImage for the SOURCE render target
        let Some(source_gpu_image) = gpu_images.get(&source_target.handle) else {
            warn!("Source texture not ready on GPU");
            return Ok(());
        };

        // Get the GpuImage for the DESTINATION render target
        let Some(destination_gpu_image) = gpu_images.get(&destination_target.handle) else {
            warn!("Destination texture not ready on GPU");
            return Ok(());
        };

        // Log texture sizes for debugging
        info!(
            "Processing background textures - Source: {}x{}, Destination: {}x{}",
            source_gpu_image.size.x,
            source_gpu_image.size.y,
            destination_gpu_image.size.x,
            destination_gpu_image.size.y
        );

        // Set up source and destination views for ping-pong
        let source_view = &source_gpu_image.texture_view;
        let destination_view = &destination_gpu_image.texture_view;

        let bind_group = render_context.render_device().create_bind_group(
            "background_lut_pingpong_bind_group",
            &background_lut_pipeline.layout,
            &BindGroupEntries::sequential((
                source_view,                             // @binding(0) background render target texture
                &background_lut_pipeline.source_sampler, // @binding(1) background render target sampler
                &lut_gpu_image.texture_view,             // @binding(2) LUT texture view
                &background_lut_pipeline.lut_sampler,    // @binding(3) LUT sampler
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("background_lut_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: destination_view, // Write to the *processed* render target
                resolve_target: None,
                ops: Operations {
                    // Clear the destination target before writing the processed background.
                    load: bevy::render::render_resource::LoadOp::Clear(Default::default()),
                    store: bevy::render::render_resource::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None, // No depth needed
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
struct BackgroundLutPipeline {
    layout: BindGroupLayout,
    source_sampler: Sampler,
    lut_sampler: Sampler, // Sampler specific for the LUT
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for BackgroundLutPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // Layout: Screen Texture, Screen Sampler, LUT Texture, LUT Sampler
        // Note: Binding 2 (Uniforms) is skipped here as we assume no dynamic settings
        let layout = render_device.create_bind_group_layout(
            "background_lut_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }), // Binding 0: Source Texture (Background Target)
                    sampler(SamplerBindingType::Filtering), // Binding 1: Source Sampler
                    texture_2d(TextureSampleType::Float { filterable: true }), // Binding 3: LUT Texture
                    sampler(SamplerBindingType::Filtering), // Binding 4: LUT Sampler
                ),
            ),
        );

        // Samplers (adjust filtering as needed)
        let source_sampler = render_device.create_sampler(&SamplerDescriptor::default());
        let lut_sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("background_lut_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        // Use the *same* shader, just bind different textures/samplers
        // If the background LUT requires different logic, create a separate shader file.
        let shader = world.load_asset(SHADER_ASSET_PATH); // Reuse existing shader

        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("background_lut_pipeline".into()),
                    layout: vec![layout.clone()],
                    vertex: fullscreen_shader_vertex_state(),
                    fragment: Some(FragmentState {
                        shader,
                        shader_defs: vec![],
                        entry_point: "fragment".into(), // Use the same entry point
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::bevy_default(), // Match render target format
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    push_constant_ranges: vec![],
                    zero_initialize_workgroup_memory: false,
                });

        Self {
            layout,
            source_sampler,
            lut_sampler,
            pipeline_id,
        }
    }
}
