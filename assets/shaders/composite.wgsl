// Nightdrawn-Tower-Defense/client/assets/shaders/composite.wgsl
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var main_texture: texture_2d<f32>;       // Source (foreground)
@group(0) @binding(1) var main_sampler: sampler;
@group(0) @binding(2) var background_texture: texture_2d<f32>; // Destination (background)
@group(0) @binding(3) var background_sampler: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Get the colors from both textures
    let src = textureSample(main_texture, main_sampler, in.uv);       // Main camera output (foreground)
    let dst = textureSample(background_texture, background_sampler, in.uv); // Background camera output

    // Standard "over" compositing: result = src + dst * (1 - src.a)
    // Assumes colors are NOT premultiplied by alpha.
    // Note: Some sources define "over" as C_out = C_src * A_src + C_dst * (1 - A_src).
    // However, for non-premultiplied alpha, the common formula is simply:
    // C_out = C_src + C_dst * (1 - A_src)
    // A_out = A_src + A_dst * (1 - A_src)
    // Blend foreground (src) over background (dst) based on foreground alpha
    let result_rgb = mix(dst.rgb, src.rgb, src.a);
    let result_a = src.a + dst.a * (1.0 - src.a);

    // Return the composited color and alpha
    return vec4<f32>(result_rgb, result_a);
}
