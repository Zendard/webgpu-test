use cgmath::{vec3, InnerSpace, Vector3};

use crate::rendering::block::State;

pub struct Chunk {
    pub position: (i32, i32),
    pub block_amount: u32,
    pub face_amount: u32,
    block_buffer: wgpu::Buffer,
    face_buffer: wgpu::Buffer,
    pub gradient_vectors: [Vector3<f32>; 8],
}

pub fn generate(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    terrain_gen_pipeline: &wgpu::ComputePipeline,
    generation_setup: &crate::rendering::block::GenerationSetup,
    chunk_position: (i32, i32),
    gradient_vectors: [Option<Vector3<f32>>; 8],
) -> Chunk {
    queue.write_buffer(
        &generation_setup.state_buffer,
        std::mem::offset_of!(State, chunk_position) as u64,
        bytemuck::cast_slice(&[chunk_position.0, chunk_position.1]),
    );
    println!("Generating and copying gradient vectors...");
    let gradient_vectors =
        copy_gradient_vectors(queue, &generation_setup.state_buffer, gradient_vectors);
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
        gradient_vectors,
    }
}

fn copy_gradient_vectors(
    queue: &wgpu::Queue,
    state_buffer: &wgpu::Buffer,
    gradient_vectors: [Option<Vector3<f32>>; 8],
) -> [Vector3<f32>; 8] {
    let all_vectors: [Vector3<f32>; 8] =
        std::array::from_fn(|i| gradient_vectors[i].unwrap_or_else(random_unit_vec3));
    dbg!(&all_vectors);
    let raw_vectors: [f32; 24] = std::array::from_fn(|i| all_vectors[i / 3].as_slice()[i % 3]);
    dbg!(&raw_vectors);

    queue.write_buffer(
        state_buffer,
        std::mem::offset_of!(State, gradient_vectors) as u64,
        bytemuck::cast_slice(&raw_vectors),
    );
    all_vectors
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
