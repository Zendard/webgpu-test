pub fn generate(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    terrain_gen_pipeline: &wgpu::ComputePipeline,
    block_bind_group: &wgpu::BindGroup,
) {
    println!("Dispatching terrain generation...");
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Terrain gen Encoder"),
    });

    let mut terrain_gen_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("Terrain gen Pass"),
        timestamp_writes: None,
    });

    terrain_gen_pass.set_pipeline(terrain_gen_pipeline);
    terrain_gen_pass.set_bind_group(0, block_bind_group, &[]);
    terrain_gen_pass.dispatch_workgroups(1, 1, 32);

    drop(terrain_gen_pass);
    queue.submit(std::iter::once(encoder.finish()));
    println!("Done");
}
