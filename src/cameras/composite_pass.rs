use bevy::{
    core_pipeline::{
        core_2d::graph::{Core2d, Node2d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    ecs::query::QueryItem,
    log::{error, info},
    prelude::*,
    render::{
        RenderApp,
        render_asset::RenderAssets,
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, FragmentState, MultisampleState, Operations,
            PipelineCache, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
            TextureFormat, TextureSampleType,
            binding_types::{sampler, texture_2d},
        },
        renderer::{RenderContext, RenderDevice},
        texture::GpuImage,
        view::ViewTarget,
    },
};

use super::background_camera::BackgroundProcessedRenderTarget;

// Original shader
// const COMPOSITE_SHADER_PATH: &str = "shaders/composite.wgsl";
// Debug version with split screen visualization
const COMPOSITE_SHADER_PATH: &str = "shaders/composite_debug.wgsl";

pub struct CompositePlugin;

impl Plugin for CompositePlugin {
    fn build(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_render_graph_node::<ViewNodeRunner<CompositeNode>>(Core2d, CompositeLabel)
            // Define edges: Composite runs after main PP and background LUT, but before Tonemapping
            // .add_render_graph_edge(
            //     Core2d,
            //     crate::cameras::shader_pipeline::post_processing::PostProcessLabel, // Depends on main post-process
            //     CompositeLabel,
            // )
            .add_render_graph_edge(
                Core2d,
                bevy::core_pipeline::core_2d::graph::Node2d::EndMainPass,
                CompositeLabel,
            )
            .add_render_graph_edge(
                Core2d,
                crate::cameras::background_lut::BackgroundLutLabel, // Depends on background post-process
                CompositeLabel,
            )
            .add_render_graph_edge(
                Core2d,
                CompositeLabel, // Tonemapping runs AFTER composite
                Node2d::Tonemapping,
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<CompositePipeline>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct CompositeLabel;

#[derive(Default)]
struct CompositeNode;

impl ViewNode for CompositeNode {
    // Query the main camera's ViewTarget and ensure it's the MainCam
    type ViewQuery = (&'static ViewTarget,);

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        view_target: QueryItem<Self::ViewQuery>, // _main_cam ensures we only run for main cam
        world: &World,
    ) -> Result<(), NodeRunError> {
        info!("Running CompositeNode for view entity");

        // Get the pipeline
        let pipeline_cache = world.resource::<PipelineCache>();
        let composite_pipeline = world.resource::<CompositePipeline>();
        let Some(pipeline) = pipeline_cache.get_render_pipeline(composite_pipeline.pipeline_id)
        else {
            info!("Composite pipeline not found or not ready yet.");
            return Ok(());
        };

        // Get the background render target
        let Some(background_target_res) = world.get_resource::<BackgroundProcessedRenderTarget>()
        else {
            error!("BackgroundProcessedRenderTarget resource not found. Cannot composite.");
            return Ok(());
        };

        // Get the GPU textures
        let gpu_images = world.resource::<RenderAssets<GpuImage>>();
        let Some(background_gpu_image) = gpu_images.get(&background_target_res.handle) else {
            info!("Background render target not yet available on GPU.");
            return Ok(());
        };

        // Get source/destination textures for the main camera view
        let post_process = view_target.0.post_process_write();

        // Create the bind group with all textures
        let bind_group = render_context.render_device().create_bind_group(
            "composite_bind_group",
            &composite_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,                    // Main camera view
                &composite_pipeline.main_sampler,       // Main view sampler
                &background_gpu_image.texture_view,     // Background texture
                &composite_pipeline.background_sampler, // Background sampler
            )),
        );

        // Begin the render pass
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("composite_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination, // Write to destination
                resolve_target: None,
                ops: Operations {
                    // Clear the destination with transparent before writing the final composite.
                    load: bevy::render::render_resource::LoadOp::Clear(Default::default()),
                    store: bevy::render::render_resource::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Draw a fullscreen quad
        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
struct CompositePipeline {
    layout: BindGroupLayout,
    main_sampler: Sampler,
    background_sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for CompositePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // Create the bind group layout
        let layout = render_device.create_bind_group_layout(
            "composite_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }), // Main view texture
                    sampler(SamplerBindingType::Filtering),                    // Main view sampler
                    texture_2d(TextureSampleType::Float { filterable: true }), // Background texture
                    sampler(SamplerBindingType::Filtering),                    // Background sampler
                ),
            ),
        );

        // Create the samplers
        let main_sampler = render_device.create_sampler(&SamplerDescriptor::default());
        let background_sampler = render_device.create_sampler(&SamplerDescriptor::default());

        // Load the shader
        let shader = world.load_asset(COMPOSITE_SHADER_PATH);

        // Create the pipeline
        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("composite_pipeline".into()),
                    layout: vec![layout.clone()],
                    vertex: fullscreen_shader_vertex_state(),
                    fragment: Some(FragmentState {
                        shader,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::bevy_default(),
                            // Disable blending; the shader calculates the final value.
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
            main_sampler,
            background_sampler,
            pipeline_id,
        }
    }
}
