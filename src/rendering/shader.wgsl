@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var<storage, read_write> state: State;
@group(1) @binding(1) var<storage, read_write> blocks: array<u32, 16384>;
@group(1) @binding(2) var<storage, read_write> faces: array<u32, 16384>;
@group(2) @binding(0) var texture_sampler: sampler;
@group(2) @binding(1) var stone_texture: texture_2d<f32>;
@group(2) @binding(2) var dirt_texture: texture_2d<f32>;
@group(2) @binding(3) var moss_texture: texture_2d<f32>;

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}

struct State {
    block_amount: atomic<u32>,
    face_amount: atomic<u32>,
    gradient_vectors: array<vec3<f32>,8>
}

struct Face {
    rotation: u32,   // 6 bits
    x: u32,            // 5 bits
    y: u32,            // 7 bits
    z: u32,            // 5 bits
    texture_id: u32,      // 3 bits
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) tex_coord: vec2f,
    @location(1) texture_id: u32,
}

const FRONT_FACE_VERTICES = array<vec3<f32>,6>(
    vec3(0., 0., 0.),
    vec3(0., 1., 0.),
    vec3(1., 1., 0.),
    vec3(0., 0., 0.),
    vec3(1., 1., 0.),
    vec3(1., 0., 0.),
);
const BACK_FACE_VERTICES= array<vec3<f32>,6>(
    vec3(1., 0., 1.),
    vec3(1., 1., 1.),
    vec3(0., 1., 1.),
    vec3(1., 0., 1.),
    vec3(0., 1., 1.),
    vec3(0., 0., 1.),
);
const TOP_FACE_VERTICES = array<vec3<f32>,6>(
    vec3(0., 1., 0.),
    vec3(0., 1., 1.),
    vec3(1., 1., 1.),
    vec3(0., 1., 0.),
    vec3(1., 1., 1.),
    vec3(1., 1., 0.),
);
const BOTTOM_FACE_VERTICES = array<vec3<f32>,6>(
    vec3(1., 0., 0.),
    vec3(1., 0., 1.),
    vec3(0., 0., 1.),
    vec3(1., 0., 0.),
    vec3(0., 0., 1.),
    vec3(0., 0., 0.),
);
const LEFT_FACE_VERTICES = array<vec3<f32>,6>(
    vec3(0., 0., 1.),
    vec3(0., 1., 1.),
    vec3(0., 1., 0.),
    vec3(0., 0., 1.),
    vec3(0., 1., 0.),
    vec3(0., 0., 0.),
);
const RIGHT_FACE_VERTICES = array<vec3<f32>,6>(
    vec3(1., 0., 0.),
    vec3(1., 1., 0.),
    vec3(1., 1., 1.),
    vec3(1., 0., 0.),
    vec3(1., 1., 1.),
    vec3(1., 0., 1.),
);

const TEX_COORDS = array<vec2<f32>,6>(
    vec2(0., 0.),
    vec2(0., 1.),
    vec2(1., 1.),
    vec2(0., 0.),
    vec2(1., 1.),
    vec2(1., 0.),
);

@vertex
fn vs_main(@builtin(vertex_index) i: u32, @builtin(instance_index) face_index: u32) -> VertexOutput {
    var out: VertexOutput;
    var local_pos = vec3(0., 0., 0.);
    let face = decode_face(faces[face_index]);

    switch face.rotation{
      // 0b100000
      case 0x20{
            local_pos = FRONT_FACE_VERTICES[i];
        }
      //0b010000
      case 0x10{
            local_pos = BACK_FACE_VERTICES[i];
        }
      //0b001000
      case 0x8{
            local_pos = TOP_FACE_VERTICES[i];
        }
      //0b000100
      case 0x4 {
            local_pos = BOTTOM_FACE_VERTICES[i];
        }
      //0b000010
      case 0x2 {
            local_pos = LEFT_FACE_VERTICES[i];
        }
      //0b000001
      case 0x1 {
            local_pos = RIGHT_FACE_VERTICES[i];
        }
      default {
            return out;
        }
}

    let world_pos = local_pos + vec3f(f32(face.x), f32(face.y), f32(face.z));
    out.position = camera.view_proj * vec4f(world_pos, 1.);
    out.tex_coord = TEX_COORDS[i];
    out.texture_id = face.texture_id;
    return out;
    //return vec4f(world_pos, 1.);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4f {
    switch input.texture_id{
        case 0{
            return textureSample(stone_texture, texture_sampler, input.tex_coord);
        }
        case 1{
            return textureSample(dirt_texture, texture_sampler, input.tex_coord);
        }
        case 2{
            return textureSample(moss_texture, texture_sampler, input.tex_coord);
        }
        default{
            return vec4f(255., 0., 255., 1.);
        }
    }
}

fn decode_face(packed: u32) -> Face {
    var voxel: Face;

    // Face flags: bits 31–26
    voxel.rotation = (packed >> 20) & 0x3Fu; // 0b111111 = 6 bits

    // X position: bits 25–21
    voxel.x = (packed >> 15u) & 0x1Fu; // 0b11111 = 5 bits

    // Y position: bits 20–14
    voxel.y = (packed >> 8u) & 0x7Fu; // 0b1111111 = 7 bits

    // Z position: bits 13–9
    voxel.z = (packed >> 3u) & 0x1Fu; // 0b11111 = 5 bits

    // Type ID: bits 2–0
    voxel.texture_id = packed & 0x7u; // 0b111 = 3 bits

    return voxel;
}

// Block: ffffffxxxxxyyyyyyyzzzzzttt
// ffffff -> Front - Back - Top - Bottom - Left - Right
// xxxxx -> x position in chunk
// yyyyyyy -> y position in chunk 
// zzzzz -> z position in chunk
// ttt
// 000 -> Stone
// 001 -> Dirt
// 010 -> Moss / Grass
