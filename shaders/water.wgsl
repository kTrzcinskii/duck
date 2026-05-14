struct CameraBuffer {
    view_proj: mat4x4<f32>,
    inverse_proj: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
};

struct LightBuffer {
    position: vec4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraBuffer;
@group(0) @binding(1) var<uniform> light: LightBuffer;

@group(1) @binding(0) var normal_tex: texture_2d<f32>;
@group(1) @binding(1) var normal_sampler: sampler;
@group(1) @binding(2) var cubemap: texture_cube<f32>;
@group(1) @binding(3) var cubemap_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
};

const positions = array<vec2<f32>, 6>(
    vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(1.0, 1.0),
    vec2(-1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VertexOutput {
    let xz = positions[vid];
    let world_pos = vec3<f32>(xz.x, 0.0, xz.y);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.uv = xz * 0.5 + 0.5;
    out.world_pos = world_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let encoded = textureSample(normal_tex, normal_sampler, in.uv).xyz;
    let normal = normalize(encoded * 2.0 - 1.0);

    let cam_pos = camera.inverse_view[3].xyz;

    let light_dir = normalize(light.position.xyz - in.world_pos);
    let view_dir = normalize(cam_pos - in.world_pos);
    let reflect_dir = reflect(-light_dir, normal);

    let ambient = 0.1;
    let diffuse = max(dot(normal, light_dir), 0.0);
    let specular = pow(max(dot(view_dir, reflect_dir), 0.0), 64.0);

    let water_color = vec4<f32>(0.1, 0.3, 0.6, 1.0);
    let color = water_color * (ambient + diffuse) + light.color * specular * 0.8;

    return color;
}