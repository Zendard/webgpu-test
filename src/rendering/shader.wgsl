struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;
@group(1) @binding(0) var<storage, read_write> blocks: array<vec3<u32>, 16384>;

const FRONT_FACE_VERTICES = array<vec3<f32>,6>(
    vec3(0., 0., 0.),
    vec3(0., 1., 0.),
    vec3(1., 1., 0.),
    vec3(0., 0., 0.),
    vec3(1., 1., 0.),
    vec3(1., 0., 0.),
);
const TOP_FACE_VERTICES = array<vec3<f32>,6>(
    vec3(0., 1., 0.),
    vec3(0., 1., 1.),
    vec3(1., 1., 1.),
    vec3(0., 1., 0.),
    vec3(1., 1., 1.),
    vec3(1., 1., 0.),
);

@vertex
fn vs_main(@builtin(vertex_index) i: u32, @builtin(instance_index) block_index: u32) -> @builtin(position)vec4f {
    var local_pos = vec3(0., 0., 0.);
    if i < 6 {
        local_pos = FRONT_FACE_VERTICES[i];
    } else {
        local_pos = TOP_FACE_VERTICES[i];
    }
    let world_pos = local_pos + vec3f(blocks[block_index]);
    return camera.view_proj * vec4f(world_pos, 1.);
    //return vec4f(world_pos, 1.);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1., 0., 0., 1.);
}
