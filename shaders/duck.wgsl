struct CameraBuffer {
    view_proj: mat4x4<f32>,
    inverse_proj: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
};

struct LightBuffer {
    position: vec4<f32>,
    color: vec4<f32>,
};

struct ModelBuffer {
    model_mtx: mat4x4<f32>,
    normal_mtx: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraBuffer;
@group(0) @binding(1) var<uniform> light: LightBuffer;

@group(1) @binding(0) var<uniform> model: ModelBuffer;
@group(1) @binding(1) var duck_tex: texture_2d<f32>;
@group(1) @binding(2) var duck_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.model_mtx * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_pos;
    out.tex_coords = in.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(duck_tex, duck_sampler, in.tex_coords);
}