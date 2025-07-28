use cgmath::{vec3, InnerSpace, Vector3};

use crate::rendering::block::State;

pub struct Chunk {
    pub position: (i32, i32),
    pub block_amount: u32,
    pub face_amount: u32,
    block_buffer: wgpu::Buffer,
    face_buffer: wgpu::Buffer,
}

pub fn generate(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    terrain_gen_pipeline: &wgpu::ComputePipeline,
    generation_setup: &crate::rendering::block::GenerationSetup,
    chunk_position: (i32, i32),
) -> Chunk {
    println!("Generating and copying gradient vectors...");
    queue.write_buffer(
        &generation_setup.state_buffer,
        std::mem::offset_of!(State, chunk_position) as u64,
        bytemuck::cast_slice(&[chunk_position.0, chunk_position.1]),
    );
    copy_gradient_vectors(queue, &generation_setup.state_buffer);
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
    terrain_gen_pass.dispatch_workgroups(32, 1, 16);

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
    let data_copy = data.to_owned();
    drop(data);
    generation_setup.staging_buffer.unmap();

    let result: &State = bytemuck::from_bytes(&data_copy);
    println!("{:?}", result);

    println!("Done");
    Chunk {
        position: chunk_position,
        block_amount: result.block_amount,
        face_amount: result.face_amount,
        block_buffer: generation_setup.block_buffer.clone(),
        face_buffer: generation_setup.face_buffer.clone(),
    }
}

fn copy_gradient_vectors(queue: &wgpu::Queue, state_buffer: &wgpu::Buffer) {
    let vec0 = random_unit_vec3();
    let vec1 = random_unit_vec3();
    let vec2 = random_unit_vec3();
    let vec3 = random_unit_vec3();
    let vec4 = random_unit_vec3();
    let vec5 = random_unit_vec3();
    let vec6 = random_unit_vec3();
    let vec7 = random_unit_vec3();
    let vec_array = &[
        vec0.as_slice(),
        vec1.as_slice(),
        vec2.as_slice(),
        vec3.as_slice(),
        vec4.as_slice(),
        vec5.as_slice(),
        vec6.as_slice(),
        vec7.as_slice(),
    ];
    queue.write_buffer(
        state_buffer,
        std::mem::offset_of!(State, gradient_vectors) as u64,
        bytemuck::cast_slice(vec_array),
    );
}

fn random_unit_vec3() -> Vector3<f32> {
    let vector: Vector3<f32> = vec3(rand::random(), rand::random(), rand::random());
    vector.normalize()
}

trait AsF32Slice {
    fn as_slice(&self) -> [f32; 3];
}
impl AsF32Slice for Vector3<f32> {
    fn as_slice(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}
