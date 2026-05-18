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

const refraction_air_to_water: f32 = 1.0 / 1.33;
const refraction_water_to_air: f32 = 1.33;
const fresnel_coefficient: f32 = 0.14;

const blue_addition: vec3<f32> = vec3<f32>(0.0, 0.05, 0.085);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let encoded = textureSample(normal_tex, normal_sampler, in.uv).xyz;
    let decoded_normal = normalize(encoded * 2.0 - 1.0);

    var normal = decoded_normal;

    let cam_pos = camera.inverse_view[3].xyz;
    let view_dir = normalize(cam_pos - in.world_pos);

    var eta = refraction_air_to_water;
    if dot(normal, view_dir) < 0.0 {
        normal = -normal;
        eta = refraction_water_to_air;
    }

    let reflect_dir = reflect(-view_dir, normal);
    let refract_dir = refract(-view_dir, normal, eta);

    let color = water_color(in.world_pos, reflect_dir, refract_dir, normal, view_dir) + blue_addition;

    return phong(vec4<f32>(color, 1.0), decoded_normal, in.world_pos, view_dir);
}

fn water_color(world_pos: vec3<f32>, reflect_dir: vec3<f32>, refract_dir: vec3<f32>, normal: vec3<f32>, view_dir: vec3<f32>) -> vec3<f32> {
    var color: vec3<f32>;
    let reflected_coords = intersect_ray(world_pos, reflect_dir);

    if length(refract_dir) < 0.001 {
        color = textureSample(cubemap, cubemap_sampler, reflected_coords).rgb;
    } else {
        let refracted_coords = intersect_ray(world_pos, refract_dir);
        let reflected_color = textureSample(cubemap, cubemap_sampler, reflected_coords).rgb;
        let refracted_color = textureSample(cubemap, cubemap_sampler, refracted_coords).rgb;

        let f = fresnel(normal, view_dir);
        color = mix(refracted_color, reflected_color, f);
    }

    return color;
}

const ambient_coefficient = 0.1;
const diffuse_coefficient = 0.8;
const specular_coefficient = 0.8;

fn phong(water_color: vec4<f32>, normal: vec3<f32>, world_pos: vec3<f32>, view_dir: vec3<f32>) -> vec4<f32> {
    let light_dir = normalize(light.position.xyz - world_pos);
    let light_reflect_dir = reflect(-light_dir, normal);

    let ambient = ambient_coefficient;
    let diffuse = max(dot(normal, light_dir), 0.0) * diffuse_coefficient;
    let specular = pow(max(dot(view_dir, light_reflect_dir), 0.0), 64.0) * specular_coefficient;

    return water_color * (ambient + diffuse) + light.color * specular;
}

fn intersect_ray(origin: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    var t_min = 1e10;

    for (var axis = 0; axis < 3; axis++) {
        let d = dir[axis];
        if abs(d) > 0.0001 {
            let t1 = (1.0 - origin[axis]) / d;
            let t2 = (-1.0 - origin[axis]) / d;
            if t1 > 0.0001 { t_min = min(t_min, t1); }
            if t2 > 0.0001 { t_min = min(t_min, t2); }
        }
    }

    return origin + t_min * dir;
}

fn fresnel(normal: vec3<f32>, view_dir: vec3<f32>) -> f32 {
    let cos_theta = max(dot(normal, view_dir), 0.0);
    return fresnel_coefficient + (1.0 - fresnel_coefficient) * pow(1.0 - cos_theta, 5.0);
}
