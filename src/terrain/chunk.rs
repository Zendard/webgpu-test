use crate::rendering::block::Block;
use std::{collections::HashSet, num::NonZeroU64};
use wgpu::util::DeviceExt;

pub const CHUNK_SIZE: u8 = 32;

#[derive(Debug)]
pub struct Chunk {
    pub position: (i32, i32),
    pub blocks: HashSet<Block>,
    instance_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_instances: u32,
}

impl Chunk {
    pub fn new(device: &wgpu::Device, position: (i32, i32), seed: u32) -> Self {
        let start = (
            position.0 * CHUNK_SIZE as i32,
            0,
            position.1 * CHUNK_SIZE as i32,
        );
        let end = (
            (position.0 + 1) * CHUNK_SIZE as i32,
            128,
            (position.1 + 1) * CHUNK_SIZE as i32,
        );

        let blocks: HashSet<Block> = super::generate_terrain(start, end, seed)
            .iter()
            .copied()
            .collect();

        let instance_data: Vec<_> = blocks
            .iter()
            .flat_map(Block::as_instances)
            .map(|instance| instance.as_raw())
            .collect();

        let block_data: Vec<[u32; 4]> = blocks
            .iter()
            .filter(|block| block.visible_faces > 0)
            .map(|block| {
                [
                    block.position.0 as u32,
                    block.position.1 as u32,
                    block.position.2 as u32,
                    0,
                ]
            })
            .collect();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!(
                "Chunk {}, {} instance buffer",
                position.0, position.1
            )),
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&instance_data),
        });

        let block_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!(
                "Chunk {}, {} block buffer",
                position.0, position.1
            )),
            usage: wgpu::BufferUsages::STORAGE,
            contents: bytemuck::cast_slice(&block_data),
        });

        let chunk_position_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!(
                "Chunk {}, {} position buffer",
                position.0, position.1
            )),
            usage: wgpu::BufferUsages::UNIFORM,
            contents: bytemuck::cast_slice(&[position.0, position.1]),
        });

        let bind_group_layout_descriptor: wgpu::BindGroupLayoutDescriptor =
            wgpu::BindGroupLayoutDescriptor {
                label: Some("Chunk bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        count: None,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        count: None,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                ],
            };

        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_descriptor);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Chunk {}, {} bind group", position.0, position.1)),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        offset: 0,
                        size: NonZeroU64::new(std::mem::size_of::<[i32; 2]>() as u64),
                        buffer: &chunk_position_buffer,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        offset: 256,
                        size: None,
                        buffer: &block_buffer,
                    }),
                },
            ],
        });

        Self {
            position,
            blocks,
            instance_buffer,
            bind_group,
            num_instances: instance_data.len() as u32,
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(1, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw_indexed(
            0..crate::rendering::block::FACE_INDICES.len() as u32,
            0,
            0..self.num_instances,
        );
    }
}
