// use wgpu::{util::DeviceExt, Device};
//
// use crate::window::Vertex;
//
// #[derive(Debug)]
// pub struct Particle {
//     x: f32,
//     y: f32,
//     speed: f32,
// }
//
// impl Particle {
//     pub fn new(x: f32, y: f32, speed: f32) -> Self {
//         Self { x, y, speed }
//     }
// }
//
// pub trait DrawParticle {
//     fn draw_particle(&mut self, device: &Device, particle: &Particle);
// }
//
// const PARTICLE_VERTICES: &[Vertex] = &[
//     Vertex {
//         position: [0.0, 1.0, 0.0],
//         color: [1.0, 0.8784313, 0.5098039],
//     },
//     Vertex {
//         position: [0.0, 0.0, 0.0],
//         color: [1.0, 0.8784313, 0.5098039],
//     },
//     Vertex {
//         position: [1.0, 0.0, 0.0],
//         color: [1.0, 0.8784313, 0.5098039],
//     },
//     Vertex {
//         position: [1.0, 1.0, 0.0],
//         color: [1.0, 0.8784313, 0.5098039],
//     },
// ];
// const PARTICLE_INDICES: &[u16] = &[
//     0, 1, 3, //Polygon0
//     1, 2, 3, //Polygon1
// ];
// impl<'a> DrawParticle for wgpu::RenderPass<'a> {
//     fn draw_particle(&mut self, device: &Device, particle: &Particle) {
//         let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Vertex Buffer"),
//             contents: bytemuck::cast_slice(PARTICLE_VERTICES),
//             usage: wgpu::BufferUsages::VERTEX,
//         });
//
//         let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Index Buffer"),
//             contents: bytemuck::cast_slice(PARTICLE_INDICES),
//             usage: wgpu::BufferUsages::INDEX,
//         });
//
//         self.set_vertex_buffer(0, vertex_buffer.slice(..));
//         self.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
//         self.draw_indexed(0..PARTICLE_INDICES.len() as u32, 0, 0..1);
//     }
// }
