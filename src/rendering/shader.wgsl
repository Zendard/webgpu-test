@group(0) @binding(0) var<storage, read_write> blocks: array<vec3<u32>, 256>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position)vec4f {
    return vec4f(vec3f(blocks[i]), 1.);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1., 0., 0., 1.);
}
