@group(0) @binding(0) var<storage, read_write> blocks: array<vec3<u32>, 256>;

@compute @workgroup_size(16,1,16)
fn gen_main(
    @builtin(global_invocation_id) block_position: vec3<u32>
) {
    blocks[block_position.x] = block_position;
}
