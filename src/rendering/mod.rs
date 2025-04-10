use crate::terrain;
use pollster::FutureExt;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use vertex::Vertex;
use wgpu::util::DeviceExt;
use wgpu::Queue;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

pub mod block;
pub mod camera;
mod hardware;
mod texture;
pub mod vertex;

const RENDER_DISTANCE: u32 = 5;
const _RENDERED_CHUNKS: u64 = (RENDER_DISTANCE as u64 * 2 - 1).pow(2);
pub const MAX_BLOCKS_IN_CHUNK: u64 = (crate::terrain::CHUNK_SIZE as u64).pow(2) * 128;

pub struct StateApplication<'a> {
    pub state: Option<State<'a>>,
    pub seed: u32,
}

impl<'a> StateApplication<'a> {
    pub fn new(seed: u32) -> Self {
        StateApplication { state: None, seed }
    }
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
    blocks: Arc<Mutex<HashSet<block::Block>>>,
    previous_chunk: (i32, i32),
    seed: u32,
    last_render_time: Instant,
    last_tick_time: Instant,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    diffuse_texture: texture::Texture,
    depth_texture: texture::RawTexture,

    instances: Arc<Mutex<Vec<Instance>>>,
    instance_buffer: wgpu::Buffer,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    async fn new(window: Window, seed: u32) -> State<'a> {
        let window = Arc::new(window);
        let window_clone = window.clone();
        let (device, config, queue, surface) = hardware::init(window_clone).await;

        let diffuse_bytes = include_bytes!("../textures/stone.png");
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
            crate::movement::PlayerController::new([16., 120., 16.], &config, &device);
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

        let terrain = Arc::new(Mutex::new(HashSet::new()));
        let mut handles = Vec::new();
        println!("Generating terrain...");
        for x in -(RENDER_DISTANCE as i32) + 1..RENDER_DISTANCE as i32 {
            for y in -(RENDER_DISTANCE as i32) + 1..RENDER_DISTANCE as i32 {
                let terrain = terrain.clone();
                let handle = std::thread::spawn(move || {
                    let chunk = terrain::generate_chunk((x, y), seed);
                    terrain.lock().unwrap().extend(chunk);
                });
                handles.push(handle);
            }
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let instances: Vec<Instance> = terrain
            .lock()
            .unwrap()
            .iter()
            .flat_map(block::Block::as_instances)
            .collect();
        let instance_data = instances.iter().map(Instance::as_raw).collect::<Vec<_>>();

        let buffer_size = 40000000;

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            mapped_at_creation: false,
            size: buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        println!("\nCopying to GPU...");
        queue.write_buffer(&instance_buffer, 0, bytemuck::cast_slice(&instance_data));
        let instances = Arc::new(Mutex::new(instances));
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
            blocks: terrain,
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
        let previous_chunk = self.previous_chunk;
        let seed = self.seed;
        let blocks = self.blocks.clone();
        let instances = self.instances.clone();
        let queue = self.queue.clone();
        let instance_buffer = self.instance_buffer.clone();

        self.previous_chunk = current_chunk;

        std::thread::spawn(move || {
            State::generate_new_chunks(
                current_chunk,
                previous_chunk,
                seed,
                blocks,
                instances,
                queue,
                instance_buffer,
            )
        });
    }

    fn generate_new_chunks(
        current_chunk: (i32, i32),
        previous_chunk: (i32, i32),
        seed: u32,
        blocks: Arc<Mutex<HashSet<block::Block>>>,
        instances: Arc<Mutex<Vec<Instance>>>,
        queue: Queue,
        buffer: wgpu::Buffer,
    ) {
        let mut chunks_to_render: Vec<(i32, i32)> =
            Vec::with_capacity((RENDER_DISTANCE * 2 - 1) as usize);
        if current_chunk.0 > previous_chunk.0 {
            for z in -(RENDER_DISTANCE as i32) - 1..RENDER_DISTANCE as i32 {
                chunks_to_render.push((
                    (current_chunk.0 + RENDER_DISTANCE as i32 - 1),
                    current_chunk.1 + z,
                ));
            }
        } else if current_chunk.0 < previous_chunk.0 {
            for z in -(RENDER_DISTANCE as i32) - 1..RENDER_DISTANCE as i32 {
                chunks_to_render.push((
                    (current_chunk.0 - RENDER_DISTANCE as i32 + 1),
                    current_chunk.1 + z,
                ));
            }
        } else if current_chunk.1 > previous_chunk.1 {
            for x in -(RENDER_DISTANCE as i32) - 1..RENDER_DISTANCE as i32 {
                chunks_to_render.push((
                    current_chunk.1 + x,
                    (current_chunk.0 + RENDER_DISTANCE as i32 - 1),
                ));
            }
        } else {
            for x in -(RENDER_DISTANCE as i32) - 1..RENDER_DISTANCE as i32 {
                chunks_to_render.push((
                    current_chunk.1 + x,
                    (current_chunk.0 - RENDER_DISTANCE as i32 + 1),
                ));
            }
        };
        dbg!(chunks_to_render.len());

        let mut terrain = HashSet::new();
        println!("Generating new chunks...");
        for chunk_to_render in chunks_to_render {
            let chunk = terrain::generate_chunk(chunk_to_render, seed);
            terrain.extend(&chunk);
        }

        let mut blocks = blocks.lock().unwrap();
        let mut instances = instances.lock().unwrap();

        let new_blocks: HashSet<block::Block> = blocks.union(&terrain).copied().collect();
        let middle_blocks: HashSet<block::Block> =
            blocks.intersection(&new_blocks).copied().collect();
        *blocks = middle_blocks.union(&terrain).copied().collect();

        *instances = blocks.iter().flat_map(block::Block::as_instances).collect();
        let instance_data = instances.iter().map(Instance::as_raw).collect::<Vec<_>>();
        println!("Copying to GPU...");
        queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&instance_data));
    }

    pub fn update(&mut self, dt: Duration) {
        self.player_controller.controller.update_camera(
            &mut self.player_controller.camera,
            &self.blocks.lock().unwrap(),
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
        let instances = self.instances.lock().unwrap();

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

        render_pass.draw_indexed(0..self.num_indices, 0, 0..instances.len() as _);
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
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
