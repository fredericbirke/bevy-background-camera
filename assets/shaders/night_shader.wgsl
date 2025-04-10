#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

// --- Bindings ---
@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var lut_texture: texture_2d<f32>;
@group(0) @binding(3) var lut_sampler: sampler;


// --- LUT Sampling Logic ---

// Define LUT properties
const LUT_DIM: f32 = 32.0; // Dimension of the LUT cube (e.g., 32x32x32)
// Texture atlas dimensions (width = LUT_DIM * LUT_DIM, height = LUT_DIM)
const ATLAS_WIDTH: f32 = 1024.0; // 32 * 32
const ATLAS_HEIGHT: f32 = 32.0; // 32

// Function to get 2D UV from 3D integer coordinates (ix, iy, iz)
fn get_uv(coords: vec3<i32>) -> vec2<f32> {
    let slice_z = f32(coords.z);
    // Calculate horizontal offset based on Z slice
    let u_offset = slice_z * LUT_DIM;
    // Calculate final U and V
    let u = (f32(coords.x) + 0.5 + u_offset) / ATLAS_WIDTH;
    let v = (f32(coords.y) + 0.5) / ATLAS_HEIGHT;
    return vec2<f32>(u, v);
}

// Function to sample the 3D LUT encoded in a 2D texture atlas
// using trilinear interpolation.
fn sample_lut_trilinear(color: vec3<f32>) -> vec3<f32> {
    // Input color is assumed to be in [0, 1] range

    // Calculate normalized 3D coordinates within the LUT cube
    // Add a small offset (half texel) to sample cell centers
    let half_texel = 0.5 / LUT_DIM;
    let lut_coords = color * (LUT_DIM - 1.0) / LUT_DIM + half_texel;

    // Ensure coordinates are within valid range [0 + half_texel, 1 - half_texel]
    // This prevents sampling outside the LUT's intended color space due to clamping/precision.
    // Clamping to the edge via sampler helps, but this ensures correct interpolation math.
    let clamped_lut_coords = clamp(lut_coords, vec3(half_texel), vec3(1.0 - half_texel));

    // Calculate the base index (floor) and fractional part (interpolation weights)
    let virtual_coords = clamped_lut_coords * LUT_DIM; // Coords in virtual 3D texels [0.5, 31.5]
    let base_coords_f = floor(virtual_coords - 0.5); // Integer part [0, 30]
    let fract_coords = virtual_coords - (base_coords_f + 0.5); // Fractional part [0, 1] for interpolation

    let base_coords_i = vec3<i32>(base_coords_f);

    // Calculate 2D texture coordinates for the 8 corner samples in the atlas


    // Sample the 8 corners
    let c000 = textureSampleLevel(lut_texture, lut_sampler, get_uv(base_coords_i + vec3(0, 0, 0)), 0.0).rgb;
    let c100 = textureSampleLevel(lut_texture, lut_sampler, get_uv(base_coords_i + vec3(1, 0, 0)), 0.0).rgb;
    let c010 = textureSampleLevel(lut_texture, lut_sampler, get_uv(base_coords_i + vec3(0, 1, 0)), 0.0).rgb;
    let c110 = textureSampleLevel(lut_texture, lut_sampler, get_uv(base_coords_i + vec3(1, 1, 0)), 0.0).rgb;
    let c001 = textureSampleLevel(lut_texture, lut_sampler, get_uv(base_coords_i + vec3(0, 0, 1)), 0.0).rgb;
    let c101 = textureSampleLevel(lut_texture, lut_sampler, get_uv(base_coords_i + vec3(1, 0, 1)), 0.0).rgb;
    let c011 = textureSampleLevel(lut_texture, lut_sampler, get_uv(base_coords_i + vec3(0, 1, 1)), 0.0).rgb;
    let c111 = textureSampleLevel(lut_texture, lut_sampler, get_uv(base_coords_i + vec3(1, 1, 1)), 0.0).rgb;

    // Trilinear interpolation
    // Interpolate along X
    let c00 = mix(c000, c100, fract_coords.x);
    let c10 = mix(c010, c110, fract_coords.x);
    let c01 = mix(c001, c101, fract_coords.x);
    let c11 = mix(c011, c111, fract_coords.x);
    // Interpolate along Y
    let c0 = mix(c00, c10, fract_coords.y);
    let c1 = mix(c01, c11, fract_coords.y);
    // Interpolate along Z
    let result = mix(c0, c1, fract_coords.z);

    return result;
}


// --- Fragment Shader Entry Point ---
@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let original_color = textureSample(screen_texture, screen_sampler, in.uv);
        // Apply the LUT
        let lut_result_rgb = sample_lut_trilinear(original_color.rgb);
        // Combine LUT RGB with original alpha
        return vec4<f32>(lut_result_rgb, original_color.a);
}
