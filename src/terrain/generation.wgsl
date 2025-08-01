@group(0) @binding(0) var<storage, read_write> state: State;
@group(0) @binding(1) var<storage, read_write> blocks: array<u32, 16384>;
@group(0) @binding(2) var<storage, read_write> faces: array<u32, 16384>;

// Gradient vectors:
//      6---------7
//     /|        /|
//    2-------- 3 |
//    | |       | |
//    | 4-------|-5
//    |/        |/
//    0---------1
 

struct State {
    block_amount: atomic<u32>,
    face_amount: atomic<u32>,
    chunk_position: vec2<i32>,
    gradient_vectors: array<vec3<f32>,8>,
}

const U32_MAX:f32 = 4294967296;
const PERLIN_NOISE_OFFSET:vec3<i32> = vec3(0, -64, 0);
const CHUNK_SIZE:vec3<u32> = vec3(32, 128, 32);

@compute @workgroup_size(1,128,2)
fn gen_main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let global_id_signed = vec3<i32>(global_id);
  // Return if the block doesn't exist
    if !check_block(global_id_signed) {
        return;
    }
    let index = atomicAdd(&state.block_amount, 1u);
    var block: u32 = 0;

  // Add faces
    if !check_block(global_id_signed + vec3<i32>(0, 0, -1)) {
        block |= 1 << 25;

        let face: u32 = (index << 3) | 0;
        let face_index = atomicAdd(&state.face_amount, 1u);
        faces[face_index] = face;
    }
    if !check_block(global_id_signed + vec3<i32>(0, 0, 1)) {
        block |= 1 << 24;

        let face: u32 = (index << 3) | 1;
        let face_index = atomicAdd(&state.face_amount, 1u);
        faces[face_index] = face;
    }
    if !check_block(global_id_signed + vec3<i32>(0, 1, 0)) {
        block |= 1 << 23;
        block |= 2; // Add moss texture

        let face: u32 = (index << 3) | 2;
        let face_index = atomicAdd(&state.face_amount, 1u);
        faces[face_index] = face;
    } else if !check_block(global_id_signed + vec3<i32>(0, 3, 0)) {
        block |= 1; // Add dirt texture
    }
    if !check_block(global_id_signed + vec3<i32>(0, -1, 0)) {
        block |= 1 << 22;

        let face: u32 = (index << 3) | 3;
        let face_index = atomicAdd(&state.face_amount, 1u);
        faces[face_index] = face;
    }
    if !check_block(global_id_signed + vec3<i32>(-1, 0, 0)) {
        block |= 1 << 21;

        let face: u32 = (index << 3) | 4;
        let face_index = atomicAdd(&state.face_amount, 1u);
        faces[face_index] = face;
    }
    if !check_block(global_id_signed + vec3<i32>(1, 0, 0)) {
        block |= 1 << 20;

        let face: u32 = (index << 3) | 5;
        let face_index = atomicAdd(&state.face_amount, 1u);
        faces[face_index] = face;
    }

  // Add positions
    block |= (global_id.x & 0x1F) << 15; // 0b11111
    block |= (global_id.y & 0x7F) << 8;  // 0b1111111
    block |= (global_id.z & 0x1F) << 3;  // 0b11111
    blocks[index] = block;
    //blocks[index] = perlin_noise(global_id_signed);
    //let block_debug = unpack_block_data(block) ;
    //blocks[state.block_amount] = block_debug;
}

fn check_block(pos: vec3<i32>) -> bool {
    let noise_value = perlin_noise(pos);
    return noise_value > 0;
}

fn perlin_noise(pos: vec3<i32>) -> f32 {
    let pos_adjusted = pos + PERLIN_NOISE_OFFSET;
    let pos_float = vec3<f32>(f32(pos_adjusted.x), f32(pos_adjusted.y), f32(pos_adjusted.z));
    // Vectors to edges of chunk
    let vec_0 = normalize(-pos_float);
    let vec_1 = normalize(-pos_float + vec3(f32(CHUNK_SIZE.x), 0, 0));
    let vec_2 = normalize(-pos_float + vec3(0, f32(CHUNK_SIZE.y), 0));
    let vec_3 = normalize(-pos_float + vec3(f32(CHUNK_SIZE.x), f32(CHUNK_SIZE.y), 0));
    let vec_4 = normalize(-pos_float + vec3(0, 0, f32(CHUNK_SIZE.z)));
    let vec_5 = normalize(-pos_float + vec3(f32(CHUNK_SIZE.x), 0, f32(CHUNK_SIZE.z)));
    let vec_6 = normalize(-pos_float + vec3(0, f32(CHUNK_SIZE.y), f32(CHUNK_SIZE.z)));
    let vec_7 = normalize(-pos_float + vec3(f32(CHUNK_SIZE.x), f32(CHUNK_SIZE.y), f32(CHUNK_SIZE.z)));

    let influence0 = dot(vec_0, state.gradient_vectors[0]);
    let influence1 = dot(vec_1, state.gradient_vectors[1]);
    let influence2 = dot(vec_2, state.gradient_vectors[2]);
    let influence3 = dot(vec_3, state.gradient_vectors[3]);
    let influence4 = dot(vec_4, state.gradient_vectors[4]);
    let influence5 = dot(vec_5, state.gradient_vectors[5]);
    let influence6 = dot(vec_6, state.gradient_vectors[6]);
    let influence7 = dot(vec_7, state.gradient_vectors[7]);

    let x_weight = pos_float.x / f32(CHUNK_SIZE.x);
    let y_weight = pos_float.y / f32(CHUNK_SIZE.y);
    let z_weight = pos_float.z / f32(CHUNK_SIZE.z);

    let avg_01 = lerp(influence0, influence1, x_weight);
    let avg_23 = lerp(influence2, influence3, x_weight);
    let avg_45 = lerp(influence4, influence5, x_weight);
    let avg_67 = lerp(influence6, influence7, x_weight);

    let avg_0123 = lerp(avg_01, avg_23, y_weight);
    let avg_4567 = lerp(avg_45, avg_67, y_weight);

    let avg = lerp(avg_0123, avg_4567, z_weight);

    return avg;
}

fn lerp(value_1: f32, value_2: f32, weight: f32) -> f32 {
    let weighted_1 = value_1 * (1 - weight);
    let weighted_2 = value_2 * weight;
    return weighted_1 + weighted_2;
}

// Block / Face : fffffffxxxxxyyyyyyyzzzzzttt
// ffffff -> Front - Back - Top - Bottom - Left - Right
// xxxxx -> x position in chunk
// yyyyyyy -> y position in chunk 
// zzzzz -> z position in chunk
// ttt
// 000 -> Stone
// 001 -> Dirt
// 010 -> Moss / Grass
struct Block {
    faces: u32,
    position: vec3<u32>,
}

fn unpack_block_data(packed: u32) -> Block {
    let z = (packed >> 3) & 0x1Fu;   // 5 bits
    let y = (packed >> 8) & 0x7Fu;   // 7 bits
    let x = (packed >> 15) & 0x1Fu;   // 5 bits
    let f = (packed >> 20) & 0x3Fu;   // 6 bits

    return Block(f, vec3<u32>(x, y, z));
}


