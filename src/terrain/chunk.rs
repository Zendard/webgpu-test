use wgpu::util::DeviceExt;

use crate::rendering::block::Block;

pub const CHUNK_SIZE: u32 = 32;
const BIND_GROUP_LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor =
    wgpu::BindGroupLayoutDescriptor {
        label: Some("Chunk bind group layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            count: None,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        }],
    };

#[derive(Debug)]
pub struct Chunk {
    pub position: (i32, i32),
    pub blocks: Vec<Block>,
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

        let blocks: Vec<Block> = super::generate_terrain(start, end, seed)
            .iter()
            .copied()
            .collect();

        let instance_data: Vec<_> = blocks
            .iter()
            .flat_map(Block::as_instances)
            .map(|instance| instance.as_raw())
            .collect();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!(
                "Chunk {}, {} instance buffer",
                position.0, position.1
            )),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&instance_data),
        });

        let chunk_position_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!(
                "Chunk {}, {} position buffer",
                position.0, position.1
            )),
            usage: wgpu::BufferUsages::UNIFORM,
            contents: bytemuck::cast_slice(&[position.0, position.1]),
        });

        let bind_group_layout = device.create_bind_group_layout(&BIND_GROUP_LAYOUT_DESCRIPTOR);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Chunk {}, {} bind group", position.0, position.1)),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    offset: 0,
                    size: None,
                    buffer: &chunk_position_buffer,
                }),
            }],
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
        render_pass.set_bind_group(2, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw_indexed(
            0..crate::rendering::block::FACE_INDICES.len() as u32,
            0,
            0..self.num_instances as u32,
        );
    }
}
