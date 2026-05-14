@group(0) @binding(0) var<storage, read> current: array<f32>;
@group(0) @binding(1) var<storage, read> previous: array<f32>;
@group(0) @binding(2) var<storage, read_write> next: array<f32>;
@group(0) @binding(3) var<storage, read> damp: array<f32>;
@group(0) @binding(4) var normal_tex: texture_storage_2d<rgba8unorm, write>;

const n: u32 = 256;
const n_f32: f32 = 256.0;
const h: f32 = 2.0 / (n_f32 - 1.0); 
const c: f32 = 1.0;
const delta_t: f32 = 1 / n_f32; 

const a: f32 = c * c * delta_t * delta_t / (h * h);
const b: f32 = 2.0 - 4.0 * a;

@compute @workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let x = id.x;
    let y = id.y;
    if x >= n || y >= n {
        return;
    }

    if x == 0 || x == n - 1u || y == 0 || y == n - 1u {
        next[idx(x, y)] = 0.0;
        return;
    }

    let cur = current[idx(x, y)];
    let prev = previous[idx(x, y)];
    let up = current[idx(x, y - 1u)];
    let down = current[idx(x, y + 1u)];
    let left = current[idx(x - 1u, y)];
    let right = current[idx(x + 1u, y)];
    let d = damp[idx(x, y)];

    let new_z = 0.99 * (a * (up + down + left + right) + b * cur - prev);
    next[idx(x, y)] = new_z;

    let dzdx = (right - left) / (2.0 * h);
    let dzdy = (down - up) / (2.0 * h);
    let normal = normalize(vec3<f32>(-dzdx, 1.0, -dzdy));

    let encoded = normal * 0.5 + vec3(0.5);
    textureStore(normal_tex, vec2<i32>(i32(x), i32(y)), vec4<f32>(encoded, 1.0));
}

fn idx(x: u32, y: u32) -> u32 {
    return y * n + x;
}