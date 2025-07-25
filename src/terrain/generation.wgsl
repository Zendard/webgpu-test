@group(0) @binding(0) var<storage, read_write> state: State;
@group(0) @binding(1) var<storage, read_write> blocks: array<u32, 16384>;
@group(0) @binding(2) var<storage, read_write> faces: array<u32, 16384>;

struct State {
    block_amount: atomic<u32>,
    face_amount: atomic<u32>,
    seed: u32,
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
        block |= 1 << 26;
        let face: u32 = 1 << 26 | ((global_id.x & 0x1F) << 15) | ((global_id.y & 0x7F) << 8) | ((global_id.z & 0x1F) << 3);
        let index = atomicAdd(&state.face_amount, 1u);
        faces[index] = face;
    }
    if !check_block(global_id_signed + vec3<i32>(0, 0, 1)) {
        block |= 1 << 25;
        let face: u32 = 1 << 25 | ((global_id.x & 0x1F) << 15) | ((global_id.y & 0x7F) << 8) | ((global_id.z & 0x1F) << 3);
        let index = atomicAdd(&state.face_amount, 1u);
        faces[index] = face;
    }
    if !check_block(global_id_signed + vec3<i32>(0, 1, 0)) {
        block |= 1 << 24;
        let face: u32 = 1 << 24 | ((global_id.x & 0x1F) << 15) | ((global_id.y & 0x7F) << 8) | ((global_id.z & 0x1F) << 3);
        let index = atomicAdd(&state.face_amount, 1u);
        faces[index] = face;
    }
    if !check_block(global_id_signed + vec3<i32>(0, -1, 0)) {
        block |= 1 << 23;
        let face: u32 = 1 << 23 | ((global_id.x & 0x1F) << 15) | ((global_id.y & 0x7F) << 8) | ((global_id.z & 0x1F) << 3);
        let index = atomicAdd(&state.face_amount, 1u);
        faces[index] = face;
    }
    if !check_block(global_id_signed + vec3<i32>(-1, 0, 0)) {
        block |= 1 << 22;
        let face: u32 = 1 << 22 | ((global_id.x & 0x1F) << 15) | ((global_id.y & 0x7F) << 8) | ((global_id.z & 0x1F) << 3);
        let index = atomicAdd(&state.face_amount, 1u);
        faces[index] = face;
    }
    if !check_block(global_id_signed + vec3<i32>(1, 0, 0)) {
        block |= 1 << 21;
        let face: u32 = 1 << 21 | ((global_id.x & 0x1F) << 15) | ((global_id.y & 0x7F) << 8) | ((global_id.z & 0x1F) << 3);
        let index = atomicAdd(&state.face_amount, 1u);
        faces[index] = face;
    }

  // Add positions
    block |= (global_id.x & 0x1F) << 15; // 0b11111
    block |= (global_id.y & 0x7F) << 8;  // 0b1111111
    block |= (global_id.z & 0x1F) << 3;  // 0b11111
    let index = atomicAdd(&state.block_amount, 1u);
    blocks[index] = block;
    //let block_debug = unpack_block_data(block) ;
    //blocks[state.block_amount] = block_debug;
}

fn check_block(pos: vec3<i32>) -> bool {
    let random_vector = random_unit_vec3(pos, state.seed);
    return abs(random_vector.x) > 0.1;
}

// Hash-based PRNG using a simple 32-bit xorshift variant
fn xorshift(seed: vec3<i32>, time: u32) -> u32 {
    var x = u32(seed.x) * 374761393u + u32(seed.y) * 668265263u + u32(seed.z) * 362437u + time;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    return x;
}

// Converts a u32 to a float in [0.0, 1.0)
fn random_f32(seed: vec3<i32>, time: u32, offset: u32) -> f32 {
    let hash = xorshift(seed + vec3<i32>(i32(offset), i32(offset * 31u), i32(offset * 67u)), time);
    let top23 = hash >> 9u; // keep top 23 bits (mantissa precision)
    return f32(top23) / 8388608.0; // divide by 2^23
}

// Generates a random normalized vec3<f32>
fn random_unit_vec3(seed: vec3<i32>, time: u32) -> vec3<f32> {
    // Get three float components in [-1, 1)
    let x = random_f32(seed, time, 1u) * 2.0 - 1.0;
    let y = random_f32(seed, time, 2u) * 2.0 - 1.0;
    let z = random_f32(seed, time, 3u) * 2.0 - 1.0;

    var v = vec3<f32>(x, y, z);

    // If the vector is too small (near-zero), re-roll with a shifted seed
    if length(v) < 1e-5 {
        v = vec3<f32>(
            random_f32(seed + vec3<i32>(1), time, 4u) * 2.0 - 1.0,
            random_f32(seed + vec3<i32>(2), time, 5u) * 2.0 - 1.0,
            random_f32(seed + vec3<i32>(3), time, 6u) * 2.0 - 1.0
        );
    }

    return normalize(v);
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


