@group(0) @binding(0) var<storage, read_write> blocks: array<vec3<u32>, 16384>;

@compute @workgroup_size(32,1,2)
fn gen_main(
    @builtin(num_workgroups) num_workgroups: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_index) local_i: u32,
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let workgroup_index = workgroup_id.x + workgroup_id.y * num_workgroups.x + workgroup_id.z * num_workgroups.x * num_workgroups.y;
    let global_i = workgroup_index * (32 * 1 * 2) + local_i;

    blocks[global_i] = global_id;
}
