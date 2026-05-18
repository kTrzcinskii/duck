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
    @location(2) tangent: vec3<f32>,
    @location(3) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) world_normal: vec3<f32>,
    @location(3) world_tangent: vec3<f32>};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.model_mtx * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_pos;
    out.tex_coords = in.tex_coords;
    out.world_pos = world_pos.xyz;
    out.world_normal = normalize((model.normal_mtx * vec4<f32>(in.normal, 0.0)).xyz);
    out.world_tangent = normalize((model.normal_mtx * vec4<f32>(in.tangent, 0.0)).xyz);
    return out;
}

const PI: f32 = 3.14159265;
const alpha_x: f32 = 10.0;
const alpha_y: f32 = 0.1;
const ambient_coefficient = 0.1;
const diffuse_coefficient = 0.2;
const specular_coefficient = 0.9;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let bitangent = normalize(cross(in.world_normal, in.world_tangent));

    let camera_pos = camera.inverse_view[3].xyz;
    let view_dir = normalize(camera_pos - in.world_pos);
    let light_dir = normalize(light.position.xyz - in.world_pos);

    let tex_color = textureSample(duck_tex, duck_sampler, in.tex_coords);

    let ambient = ambient_coefficient;
    let diffuse = max(dot(in.world_normal, light_dir), 0.0) * diffuse_coefficient;
    let specular = ward_anisotropic(in.world_normal, light_dir, view_dir, in.world_tangent, bitangent) * specular_coefficient;

    return tex_color * (ambient + diffuse) + light.color * specular;
}

// https://www.graphics.cornell.edu/~bjw/wardnotes.pdf
// equation 4
fn ward_anisotropic(
    normal: vec3<f32>,
    light_dir: vec3<f32>,
    view_dir: vec3<f32>,
    tangent: vec3<f32>,
    bitangent: vec3<f32>,
) -> f32 {
    let half_vec = normalize(light_dir + view_dir);

    let normal_dot_light_dir = max(dot(normal, light_dir), 0.0);
    let normal_dot_view_dir = max(dot(normal, view_dir), 0.0);
    let normal_dot_half_vec = max(dot(normal, half_vec), 0.0001);

    if normal_dot_light_dir <= 0.0 || normal_dot_view_dir <= 0.0 {
        return 0.0;
    }

    let half_vec_dot_tangent = dot(half_vec, tangent);
    let half_vec_dot_bitangent = dot(half_vec, bitangent);

    let exponent = -((half_vec_dot_tangent * half_vec_dot_tangent) / (alpha_x * alpha_x) +
            (half_vec_dot_bitangent * half_vec_dot_bitangent) / (alpha_y * alpha_y)) / (normal_dot_half_vec * normal_dot_half_vec);

    let denom = 4.0 * PI * alpha_x * alpha_y * sqrt(normal_dot_light_dir * normal_dot_view_dir);

    return exp(exponent) / denom;
}
