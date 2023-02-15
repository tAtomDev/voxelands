#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_types

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) data: u32,
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

    var color_intensity: f32 = f32((in.data >> 2u) & 5u) / 5.0;
    if color_intensity < 0.4 {
        color_intensity = 0.4;
    }

    out.color_intensity = color_intensity;

    var uvs: vec2<f32> = vec2<f32>(
        f32((in.data >> 1u) & 1u),
        f32(in.data & 1u),
    );

    out.uvs = uvs;
    out.texture_index = (in.data >> 6u) & 255u;

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