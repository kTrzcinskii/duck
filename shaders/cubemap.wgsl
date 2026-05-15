struct CameraBuffer {
    view_proj: mat4x4<f32>,
    inverse_proj: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraBuffer;

@group(1) @binding(2) var cubemap: texture_cube<f32>;
@group(1) @binding(3) var cubemap_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) tex_cord: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_pos = camera.view_proj * vec4<f32>(input.position, 1.0);
    out.tex_cord = normalize(input.position);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(cubemap, cubemap_sampler, input.tex_cord);
}