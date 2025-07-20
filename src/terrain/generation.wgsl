@group(0) @binding(0) var<storage, read_write> blocks: array<u32, 16384>;

@compute @workgroup_size(32,1,2)
fn gen_main(
    @builtin(num_workgroups) num_workgroups: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_index) local_i: u32,
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
  // Return if de block doesn't exist
    if !check_block(global_id) {
        return;
    }

    let workgroup_index = workgroup_id.x + workgroup_id.y * num_workgroups.x + workgroup_id.z * num_workgroups.x * num_workgroups.y;
    let global_i = workgroup_index * (32 * 1 * 2) + local_i;


    var block: u32 = 0;
    if !check_block(global_id + vec3(0., 0., -1.)) {
        block |= 1 << 26;
    }
    if !check_block(global_id + vec3(0., 0., 1.)) {
        block |= 1 << 25;
    }
    if !check_block(global_id + vec3(0., 1., 0.)) {
        block |= 1 << 24;
    }
    if !check_block(global_id + vec3(0., -1., 0.)) {
        block |= 1 << 23;
    }
    if !check_block(global_id + vec3(-1., 0., 0.)) {
        block |= 1 << 22;
    }
    if !check_block(global_id + vec3(1., 0., 0.)) {
        block |= 1 << 21;
    }

  // Return if a block has no faces
    if block >> 20 == 0 {
        return;
    }
    block |= (global_id.x >> 27) << 15;
    block |= (global_id.y >> 25) << 10;
    block |= (global_id.z >> 27) << 3;
    blocks[global_i] = block;
}

fn check_block(pos: vec3<u32>) -> bool {
    return true;
}

// Block: fffffffxxxxxyyyyyyyzzzzzttt
// ffffff -> Front - Back - Top - Bottom - Left - Right
// xxxxx -> x position in chunk
// yyyyyyy -> y position in chunk 
// zzzzz -> z position in chunk
// ttt
// 000 -> Stone
// 001 -> Dirt
// 010 -> Moss / Grass
