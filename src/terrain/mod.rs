use cgmath::InnerSpace;

pub fn generate(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    terrain_gen_pipeline: &wgpu::ComputePipeline,
    generation_setup: &crate::rendering::block::GenerationSetup,
) -> u32 {
    println!("Dispatching terrain generation...");
    unsafe {
        device.start_graphics_debugger_capture();
    }
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Terrain gen Encoder"),
    });

    let mut terrain_gen_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("Terrain gen Pass"),
        timestamp_writes: None,
    });

    terrain_gen_pass.set_pipeline(terrain_gen_pipeline);
    terrain_gen_pass.set_bind_group(0, &generation_setup.bind_group, &[]);
    terrain_gen_pass.dispatch_workgroups(1, 32, 32);

    drop(terrain_gen_pass);
    encoder.copy_buffer_to_buffer(
        &generation_setup.state_buffer,
        0,
        &generation_setup.staging_buffer,
        0,
        None,
    );
    queue.submit(std::iter::once(encoder.finish()));

    println!("Reading face amount...");
    let buffer_slice = generation_setup.staging_buffer.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |result| {
        assert!(result.is_ok());
    });
    device.poll(wgpu::PollType::Wait).unwrap();

    let data = buffer_slice.get_mapped_range();

    let result: &crate::rendering::block::State = bytemuck::from_bytes(&data);
    println!("{:?}", result);
    let face_amount = result.face_amount;
    drop(data);
    generation_setup.staging_buffer.unmap();

    println!("Done");
    face_amount
}
