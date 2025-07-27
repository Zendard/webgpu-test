@group(0) @binding(0) var<storage, read_write> state: State;
@group(0) @binding(1) var<storage, read_write> blocks: array<u32, 16384>;
@group(0) @binding(2) var<storage, read_write> faces: array<u32, 16384>;

struct State {
    block_amount: atomic<u32>,
    face_amount: atomic<u32>,
    gradient_vectors: array<vec3<f32>,8>,
}

const U32_MAX:f32 = 4294967296;

@compute @workgroup_size(32,1,1)
fn gen_main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let global_id_signed = vec3<i32>(global_id);
  // Return if the block doesn't exist
    if !check_block(global_id_signed) {
        return;
    }
    var block: u32 = 0;

  // Add faces
    if !check_block(global_id_signed + vec3<i32>(0, 0, -1)) {
        block |= 1 << 25;
        let face: u32 = 1 << 25 | ((global_id.x & 0x1F) << 15) | ((global_id.y & 0x7F) << 8) | ((global_id.z & 0x1F) << 3);
        let index = atomicAdd(&state.face_amount, 1u);
        faces[index] = face;
    }
    if !check_block(global_id_signed + vec3<i32>(0, 0, 1)) {
        block |= 1 << 24;
        let face: u32 = 1 << 24 | ((global_id.x & 0x1F) << 15) | ((global_id.y & 0x7F) << 8) | ((global_id.z & 0x1F) << 3);
        let index = atomicAdd(&state.face_amount, 1u);
        faces[index] = face;
    }
    if !check_block(global_id_signed + vec3<i32>(0, 1, 0)) {
        block |= 1 << 23;
        let face: u32 = 1 << 23 | ((global_id.x & 0x1F) << 15) | ((global_id.y & 0x7F) << 8) | ((global_id.z & 0x1F) << 3);
        let index = atomicAdd(&state.face_amount, 1u);
        faces[index] = face;
    }
    if !check_block(global_id_signed + vec3<i32>(0, -1, 0)) {
        block |= 1 << 22;
        let face: u32 = 1 << 22 | ((global_id.x & 0x1F) << 15) | ((global_id.y & 0x7F) << 8) | ((global_id.z & 0x1F) << 3);
        let index = atomicAdd(&state.face_amount, 1u);
        faces[index] = face;
    }
    if !check_block(global_id_signed + vec3<i32>(-1, 0, 0)) {
        block |= 1 << 21;
        let face: u32 = 1 << 21 | ((global_id.x & 0x1F) << 15) | ((global_id.y & 0x7F) << 8) | ((global_id.z & 0x1F) << 3);
        let index = atomicAdd(&state.face_amount, 1u);
        faces[index] = face;
    }
    if !check_block(global_id_signed + vec3<i32>(1, 0, 0)) {
        block |= 1 << 20;
        let face: u32 = 1 << 20 | ((global_id.x & 0x1F) << 15) | ((global_id.y & 0x7F) << 8) | ((global_id.z & 0x1F) << 3);
        let index = atomicAdd(&state.face_amount, 1u);
        faces[index] = face;
    }

  // Add positions
    block |= (global_id.x & 0x1F) << 15; // 0b11111
    block |= (global_id.y & 0x7F) << 8;  // 0b1111111
    block |= (global_id.z & 0x1F) << 3;  // 0b11111
    let index = atomicAdd(&state.block_amount, 1u);
    blocks[index] = block;
    //blocks[index] = perlin_noise(global_id_signed);
    //let block_debug = unpack_block_data(block) ;
    //blocks[state.block_amount] = block_debug;
}

fn check_block(pos: vec3<i32>) -> bool {
    let noise_value = perlin_noise(pos);
    return noise_value > -0.6;
}

fn perlin_noise(pos: vec3<i32>) -> f32 {
    let pos_float = vec3<f32>(f32(pos.x), f32(pos.y), f32(pos.z));
    // Vectors to edges of chunk
    let vec_0 = normalize(-pos_float);
    let vec_1 = normalize(-pos_float + vec3(32, 0, 0));
    let vec_2 = normalize(-pos_float + vec3(0, 128, 0));
    let vec_3 = normalize(-pos_float + vec3(32, 128, 0));
    let vec_4 = normalize(-pos_float + vec3(0, 0, 32));
    let vec_5 = normalize(-pos_float + vec3(32, 0, 32));
    let vec_6 = normalize(-pos_float + vec3(0, 128, 32));
    let vec_7 = normalize(-pos_float + vec3(32, 128, 32));

    let influence0 = dot(vec_0, state.gradient_vectors[0]);
    let influence1 = dot(vec_0, state.gradient_vectors[1]);
    let influence2 = dot(vec_0, state.gradient_vectors[2]);
    let influence3 = dot(vec_0, state.gradient_vectors[3]);
    let influence4 = dot(vec_0, state.gradient_vectors[4]);
    let influence5 = dot(vec_0, state.gradient_vectors[5]);
    let influence6 = dot(vec_0, state.gradient_vectors[6]);
    let influence7 = dot(vec_0, state.gradient_vectors[7]);

    let x_weight = pos_float.x / 32;
    let y_weight = pos_float.y / 128;
    let z_weight = pos_float.z / 32;

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


