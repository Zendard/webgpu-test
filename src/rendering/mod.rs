use pollster::FutureExt;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::terrain;

pub mod block;
pub mod camera;
mod culling;
mod hardware;
mod texture;

pub struct StateApplication<'a> {
    pub state: Option<State<'a>>,
    pub seed: u32,
}

impl<'a> StateApplication<'a> {
    pub fn new(seed: u32) -> Self {
        StateApplication { state: None, seed }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl<'a> ApplicationHandler for StateApplication<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes().with_title("WGPU test"))
            .unwrap();
        self.state = Some(State::new(window, self.seed).block_on());
    }

    fn device_event(
        &mut self,
        _: &ActiveEventLoop,
        _: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.state
                .as_mut()
                .unwrap()
                .player_controller
                .controller
                .process_mouse(delta.0, delta.1);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let window = self.state.as_ref().unwrap().window();

        if window.id() == window_id {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    self.state.as_mut().unwrap().resize(physical_size);
                }
                WindowEvent::KeyboardInput { .. } | WindowEvent::MouseInput { .. } => {
                    let state = self.state.as_mut().unwrap();
                    state.input(&event);
                }
                WindowEvent::RedrawRequested => {
                    let state = self.state.as_mut().unwrap();

                    let dt = Instant::now() - state.last_render_time;
                    state.update(dt);
                    if (Instant::now() - state.last_tick_time) >= Duration::from_millis(50) {
                        state.tick_update();
                        state.last_tick_time = Instant::now();
                    }
                    state.render().unwrap();
                    state.last_render_time = Instant::now();
                    self.state.as_ref().unwrap().window().request_redraw();
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug)]
pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
    render_pipeline: wgpu::RenderPipeline,

    player_controller: crate::movement::PlayerController,
    blocks: HashSet<block::Block>,
    edge_blocks: HashSet<block::Block>,
    previous_chunk: (i32, i32),
    seed: u32,
    last_render_time: Instant,
    last_tick_time: Instant,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    diffuse_texture: texture::Texture,
    depth_texture: texture::RawTexture,

    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    async fn new(window: Window, seed: u32) -> State<'a> {
        let window = Arc::new(window);
        let window_clone = window.clone();
        let (device, config, queue, surface) = hardware::init(window_clone).await;

        let diffuse_bytes = include_bytes!("../textures/cobblestone.png");
        let diffuse_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            diffuse_bytes,
            "textures/cobblestone.png",
        )
        .unwrap();

        let depth_texture = texture::RawTexture::create_depth_texture(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice::<Vertex, u8>(self::block::FACE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice::<u16, u8>(self::block::FACE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = self::block::FACE_INDICES.len().try_into().unwrap();

        let mut player_controller =
            crate::movement::PlayerController::new([0., 120., 0.], &config, &device);
        player_controller
            .camera_uniform
            .update_view_proj(&player_controller.camera, &player_controller.projection);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &diffuse_texture.bind_group_layout,
                    &player_controller.camera.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::RawTexture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let (terrain, mut edge_blocks) =
            terrain::generate_terrain((-63, 0, -63), (96, 10, 96), seed);
        let blocks = terrain;
        let instances: Vec<Instance> = culling::blocks_to_instances(&blocks, &mut edge_blocks);
        let instance_data = instances.iter().map(Instance::as_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            mapped_at_creation: false,
            size: 32 * 32 * 10 * 16 * 360,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        queue.write_buffer(&instance_buffer, 0, bytemuck::cast_slice(&instance_data));
        Self {
            surface,
            device,
            queue,
            config,
            window,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_texture,
            depth_texture,
            player_controller,
            blocks,
            edge_blocks,
            previous_chunk: (0, 0),
            seed,
            last_render_time: Instant::now(),
            last_tick_time: Instant::now(),
            instances,
            instance_buffer,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::RawTexture::create_depth_texture(&self.device, &self.config);
            self.player_controller
                .projection
                .resize(new_size.width, new_size.height);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => {
                if *key == KeyCode::Escape {
                    self.window
                        .set_cursor_grab(winit::window::CursorGrabMode::None)
                        .unwrap();
                    self.window.set_cursor_visible(true);
                }
                self.player_controller
                    .controller
                    .process_keyboard(*key, *state)
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                if state.is_pressed() {
                    self.window
                        .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                        .unwrap();
                    self.window.set_cursor_visible(false);
                }
                true
            }
            _ => false,
        }
    }

    fn update_terrain(&mut self) {
        let current_chunk = (
            self.player_controller.camera.position.x as i32 / 32,
            self.player_controller.camera.position.z as i32 / 32,
        );

        // Dont need to update when in the same chunk
        if current_chunk == self.previous_chunk {
            return;
        }
        dbg!(current_chunk);

        let chunk_to_render = if current_chunk.0 > self.previous_chunk.0 {
            (current_chunk.0 + 2, current_chunk.1)
        } else if current_chunk.0 < self.previous_chunk.0 {
            (current_chunk.0 - 2, current_chunk.1)
        } else if current_chunk.1 > self.previous_chunk.1 {
            (current_chunk.0, current_chunk.1 + 2)
        } else {
            (current_chunk.0, current_chunk.1 - 2)
        };

        let (terrain, edge_blocks) = terrain::generate_terrain(
            (chunk_to_render.0 * 32 + 1, 0, chunk_to_render.1 * 32 + 1),
            (
                (chunk_to_render.0 + 1) * 32,
                10,
                (chunk_to_render.1 + 1) * 32,
            ),
            self.seed,
        );

        let new_blocks: HashSet<block::Block> =
            self.blocks.union(&terrain).map(|block| *block).collect();
        let middle_blocks: HashSet<block::Block> = self
            .blocks
            .intersection(&new_blocks)
            .map(|block| *block)
            .collect();
        self.blocks = middle_blocks.union(&terrain).map(|block| *block).collect();
        self.edge_blocks = self
            .edge_blocks
            .union(&edge_blocks)
            .map(|block| *block)
            .collect();

        self.instances = culling::blocks_to_instances(&self.blocks, &mut self.edge_blocks);
        let instance_data = self
            .instances
            .iter()
            .map(Instance::as_raw)
            .collect::<Vec<_>>();
        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        );

        self.previous_chunk = current_chunk;
    }

    pub fn update(&mut self, dt: Duration) {
        self.player_controller.controller.update_camera(
            &mut self.player_controller.camera,
            &self.blocks,
            dt,
        );

        self.update_terrain();

        self.player_controller.camera_uniform.update_view_proj(
            &self.player_controller.camera,
            &self.player_controller.projection,
        );

        self.queue.write_buffer(
            &self.player_controller.camera.buffer,
            0,
            bytemuck::cast_slice(&[self.player_controller.camera_uniform]),
        );
    }

    pub fn tick_update(&mut self) {
        self.player_controller.controller.tick_update_camera()
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, &self.diffuse_texture.bind_group, &[]);
        render_pass.set_bind_group(1, &self.player_controller.camera.bind_group, &[]);

        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    fn as_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
            .into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
