// Debug version of composite shader with visualization features
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var main_texture: texture_2d<f32>;       // Source (foreground)
@group(0) @binding(1) var main_sampler: sampler;
@group(0) @binding(2) var background_texture: texture_2d<f32>; // Destination (background)
@group(0) @binding(3) var background_sampler: sampler;

// Creates a checkerboard pattern for transparent areas
fn checkerboard(uv: vec2<f32>) -> vec3<f32> {
    let checker_size = 16.0;
    let cx = floor(uv.x * checker_size);
    let cy = floor(uv.y * checker_size);
    let result = fract((cx + cy) * 0.5) * 2.0;
    if (result < 0.5) {
        return vec3<f32>(0.2, 0.2, 0.2);
    } else {
        return vec3<f32>(0.4, 0.4, 0.4);
    }
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Get the colors from both textures
    let src = textureSample(main_texture, main_sampler, in.uv);       // Main camera output (foreground)
    let dst = textureSample(background_texture, background_sampler, in.uv); // Background camera output

    // Debug split screen approach - left side shows src alpha, right side shows normal composite
    if (in.uv.x < 0.33) {
        // Left third: Show foreground (game camera) with a checkerboard where alpha is present
        // Visualize alpha as a grayscale overlay in red channel
            return vec4<f32>(src.rgb, 1.0); // Top: Show main camera color with full opacity
    } else if (in.uv.x < 0.66) {
        // Middle third: Show background
            return vec4<f32>(dst.rgb, 1.0); // Top: Show background camera color with full opacity
    } else {
        // Right third: Standard alpha composite
        // Standard "over" compositing: result = src + dst * (1 - src.a)
        let result_rgb = mix(dst.rgb, src.rgb, src.a);
        let result_a = src.a + dst.a * (1.0 - src.a);

        return vec4<f32>(result_rgb, result_a);
    }
}
