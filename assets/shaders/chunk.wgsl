#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_types

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uvs: vec2<f32>,
    @location(2) color_intensity: f32,
    @location(3) texture_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color_intensity: f32,
    @location(1) uvs: vec2<f32>,
    @location(2) texture_index: u32,
}

@group(2) @binding(0)
var<uniform> mesh: Mesh;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = view.view_proj * mesh.model * vec4<f32>(in.position, 1.0);
    out.color_intensity = in.color_intensity;
    out.uvs = in.uvs;
    out.texture_index = in.texture_index;

    return out;
}

@group(1) @binding(0)
var texture: texture_2d_array<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color_intensity * textureSample(texture, texture_sampler, in.uvs, i32(in.texture_index));
}