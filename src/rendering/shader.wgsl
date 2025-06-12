struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>
};
@group(0) @binding(0)
var<uniform> camera: Camera;

struct AlignedArray {
    @align(16)
  value: array<u32>,
}

@group(1) @binding(0)
var<uniform> chunk: vec2<i32>;
@group(1) @binding(1)
var<storage, read> blocks: array<vec4<u32>>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) face: u32,
    @location(2) texture: u32,
    @location(3) light_amount: f32,
};

struct InstanceInput {
    @location(5) raw_description: u32,
};

const FACE_TO_SIN_COS_X = array<f32, 12>(
    0, 1,
    0, 1,
    -1, 0,
    1, 0,
    0, 1,
    0, 1,
);

const FACE_TO_SIN_COS_Y = array<f32, 12>(
    -1, 0,
    1, 0,
    0, 1,
    0, 1,
    0, -1,
    0, 1,
);

const TOP_FACE_TRANSLATION = mat4x4<f32>(
    1, 0, 0, 0,
    0, 1, 0, 0,
    0, 0, 1, 0,
    0, 0, 0, 1,
);

const FRONT_FACE_TRANSLATION = mat4x4<f32>(
    1, 0, 0, 0,
    0, 1, 0, 0,
    0, 0, 1, 0,
    -1, -1, 0, 1,
);

const BACK_FACE_TRANSLATION = mat4x4<f32>(
    1, 0, 0, 0,
    0, 1, 0, 0,
    0, 0, 1, 0,
    0, 0, 0, 1,
);

const FACE_TO_SUNLIGHT = array<f32,6>(
    3.5, 1.5, 4.5, 0.5, 2.5, 3.5
);

const SUNGLIGHT_STRENGTH = 0.3;

const LIGHT_RAY_DIR = vec3<f32>(.5, 1, .5);
const MAX_RAY_LENGTH = 32;

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let data = instance.raw_description;
    let texture = (data >> 23) & 7;
    let face = (data >> 20) & 7;
    let position_x = (data >> 14) & 63;
    let position_y = (data >> 6) & 255;
    let position_z = (data) & 63;

    let sin_x = FACE_TO_SIN_COS_X[face * 2];
    let cos_x = FACE_TO_SIN_COS_X[face * 2 + 1];
    let sin_y = FACE_TO_SIN_COS_Y[face * 2];
    let cos_y = FACE_TO_SIN_COS_Y[face * 2 + 1];

    let rotation_matrix_x = mat4x4<f32>(
        1, 0, 0, 0,
        0, cos_x, sin_x, 0,
        0, -sin_x, cos_x, 0,
        0, 0, 0, 1,
    );

    let rotation_matrix_y = mat4x4<f32>(
        cos_y, 0, -sin_y, 0,
        0, 1, 0, 0,
        sin_y, 0, cos_y, 0,
        0, 0, 0, 1,
    );

    let translation_matrix = mat4x4<f32>(
        1, 0, 0, 0,
        0, 1, 0, 0,
        0, 0, 1, 0,
        f32(position_x), f32(position_y), f32(position_z), 1,
    );

    let chunk_translation_matrix = mat4x4<f32>(
        1, 0, 0, 0,
        0, 1, 0, 0,
        0, 0, 1, 0,
        f32(chunk.x) * 32, 0, f32(chunk.y) * 32, 1,
    );


    var tranform_matrix = chunk_translation_matrix * translation_matrix * rotation_matrix_x * rotation_matrix_y;


    if face == 2 { // Top face needs to be moved up 
        tranform_matrix *= TOP_FACE_TRANSLATION;
  
    } else if face == 4 && position_y > 0 { // Front face need to be moved down 
        tranform_matrix *= FRONT_FACE_TRANSLATION;
  
    } else if face == 5 { // Back face needs to be moved back 
        tranform_matrix *= BACK_FACE_TRANSLATION;
    }

    let world_position = tranform_matrix * vec4<f32>(model.position, 1.);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_position;
    out.texture = texture;
    out.tex_coords = model.tex_coords;
    out.face = face;
    out.light_amount = calculate_light(world_position.xyz);
    return out;
}


@group(2) @binding(0)
var texture_sampler: sampler;
@group(2) @binding(1)
var stone_diffuse: texture_2d<f32>;
@group(2) @binding(2)
var dirt_diffuse: texture_2d<f32>;
@group(2) @binding(3)
var moss_diffuse: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //let face = in.color;
    //if face == 0 {
    //    return vec4<f32>(0, 0, 0, 1);
    //} else if face == 1 {
    //    return vec4<f32>(0.5, 0, 0, 1);
    //} else if face == 2 {
    //    return vec4<f32>(0, 0.5, 0, 1);
    //} else if face == 3 {
    //    return vec4<f32>(0, 0, 0.5, 1);
    //} else if face == 4 {
    //    return vec4<f32>(0.5, 0.5, 0, 1);
    //} else if face == 5 {
    //    return vec4<f32>(0, 0.5, 0.5, 1);
    //} else {
    //    return vec4<f32>(0.5, 0.5, 0.5, 1);
    //} 
    var out: vec4<f32>;
    if in.texture == 0 {
        let texture_diffuse = stone_diffuse;
        out = textureSample(texture_diffuse, texture_sampler, in.tex_coords);
    } else if in.texture == 1 {
        let texture_diffuse = dirt_diffuse;
        out = textureSample(texture_diffuse, texture_sampler, in.tex_coords);
    } else if in.texture == 2 {
        let texture_diffuse = moss_diffuse;
        out = textureSample(texture_diffuse, texture_sampler, in.tex_coords);
    }

    return out * FACE_TO_SUNLIGHT[in.face] * SUNGLIGHT_STRENGTH * in.light_amount;
}

fn calculate_light(pixel_pos: vec3<f32>) -> f32 {
    let  block_amount = i32(arrayLength(&blocks));
    for (var i = 0; i < block_amount; i++) {
        let block_position = vec3(f32(blocks[i].x), f32(blocks[i].y), f32(blocks[i].z));
        let t_low = (block_position - pixel_pos) / LIGHT_RAY_DIR;
        let t_high = (block_position + vec3(1, 1, 1) - pixel_pos) / LIGHT_RAY_DIR;
        let t_close_vec = min(t_low, t_high);
        let t_far_vec = max(t_low, t_high);
        let t_close = max(t_close_vec.x, max(t_close_vec.y, t_close_vec.z));
        let t_far = min(t_far_vec.x, min(t_far_vec.y, t_far_vec.z));

        if t_close <= t_far && sign(t_close) > 0 {
            return 0.3;
        }
    }
    return 1.;
}

// 0 -> left   -> black
// 1 -> right  -> red
// 2 -> top    -> green
// 3 -> bottom -> blue
// 4 -> front  -> yellow
// 5 -> back   -> light blue

