use std::hash::Hasher;

use crate::rendering::{
    block::{GenerationSetup, State},
    CHUNK_SIZE,
};
use cgmath::{point3, vec3, InnerSpace, Point2, Point3, Vector3};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub position: Point2<i32>,
    pub face_amount: u32,
    pub generation_setup: GenerationSetup,
}

pub fn generate(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    terrain_gen_pipeline: &wgpu::ComputePipeline,
    chunk_position: Point2<i32>,
    seed: u32,
) -> Chunk {
    let generation_setup = GenerationSetup::new(device);
    queue.write_buffer(
        &generation_setup.state_buffer,
        std::mem::offset_of!(State, chunk_position) as u64,
        bytemuck::cast_slice(&[chunk_position.x, chunk_position.y]),
    );
    println!("Generating and copying gradient vectors...");
    copy_gradient_vectors(queue, &generation_setup.state_buffer, seed, chunk_position);
    println!("Dispatching terrain generation...");

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
        face_amount: result.face_amount,
        generation_setup,
    }
}

fn copy_gradient_vectors(
    queue: &wgpu::Queue,
    state_buffer: &wgpu::Buffer,
    seed: u32,
    chunk_position: Point2<i32>,
) {
    let flat_vectors: Vec<f32> = [
        random_unit_vec3(
            point3(
                chunk_position.x * CHUNK_SIZE.x as i32,
                0,
                chunk_position.y * CHUNK_SIZE.z as i32,
            ),
            seed,
        ),
        random_unit_vec3(
            point3(
                chunk_position.x * CHUNK_SIZE.x as i32 + CHUNK_SIZE.x as i32,
                0,
                chunk_position.y * CHUNK_SIZE.z as i32,
            ),
            seed,
        ),
        random_unit_vec3(
            point3(
                chunk_position.x * CHUNK_SIZE.x as i32,
                CHUNK_SIZE.y as i32,
                chunk_position.y * CHUNK_SIZE.z as i32,
            ),
            seed,
        ),
        random_unit_vec3(
            point3(
                chunk_position.x * CHUNK_SIZE.x as i32 + CHUNK_SIZE.x as i32,
                CHUNK_SIZE.y as i32,
                chunk_position.y * CHUNK_SIZE.z as i32,
            ),
            seed,
        ),
        random_unit_vec3(
            point3(
                chunk_position.x * CHUNK_SIZE.x as i32,
                0,
                chunk_position.y * CHUNK_SIZE.z as i32 + CHUNK_SIZE.z as i32,
            ),
            seed,
        ),
        random_unit_vec3(
            point3(
                chunk_position.x * CHUNK_SIZE.x as i32 + CHUNK_SIZE.x as i32,
                0,
                chunk_position.y * CHUNK_SIZE.z as i32 + CHUNK_SIZE.z as i32,
            ),
            seed,
        ),
        random_unit_vec3(
            point3(
                chunk_position.x * CHUNK_SIZE.x as i32,
                CHUNK_SIZE.y as i32,
                chunk_position.y * CHUNK_SIZE.z as i32 + CHUNK_SIZE.z as i32,
            ),
            seed,
        ),
        random_unit_vec3(
            point3(
                chunk_position.x * CHUNK_SIZE.x as i32 + CHUNK_SIZE.x as i32,
                CHUNK_SIZE.y as i32,
                chunk_position.y * CHUNK_SIZE.z as i32 + CHUNK_SIZE.z as i32,
            ),
            seed,
        ),
    ]
    .iter()
    .flat_map(|vec3| [vec3.x, vec3.y, vec3.z])
    .collect();
    queue.write_buffer(
        state_buffer,
        std::mem::offset_of!(State, gradient_vectors) as u64,
        bytemuck::cast_slice(&flat_vectors),
    );
}

fn random_unit_vec3(position: Point3<i32>, seed: u32) -> Vector3<f32> {
    let mut hasher = std::hash::DefaultHasher::new();
    hasher.write_u32(seed);
    hasher.write_i32(position.x);
    hasher.write_i32(position.y);
    hasher.write_i32(position.z);
    let hash = hasher.finish();
    let mut rng = <rand::rngs::SmallRng as rand::SeedableRng>::seed_from_u64(hash);
    let vector: Vector3<f32> = vec3(rng.random(), rng.random(), rng.random());
    vector.normalize()
}
