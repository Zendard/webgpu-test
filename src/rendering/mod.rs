use crate::terrain::chunk::Chunk;
use block::Block;
use instance::{Instance, InstanceRaw};
use pollster::FutureExt;
use std::collections::HashMap;
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
pub mod instance;
mod texture;
pub mod vertex;

const RENDER_DISTANCE: u32 = 1;
pub const MAX_FACES_IN_CHUNK: u64 = (crate::terrain::chunk::CHUNK_SIZE as u64).pow(2) * 128 / 2 * 6;
const CHUNK_BUFFER_SIZE: u64 = MAX_FACES_IN_CHUNK * std::mem::size_of::<InstanceRaw>() as u64;

pub struct StateApplication<'a> {
    pub state: Option<State<'a>>,
    pub seed: u32,
}

impl StateApplication<'_> {
    pub fn new(seed: u32) -> Self {
        StateApplication { state: None, seed }
    }
}

impl ApplicationHandler for StateApplication<'_> {
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
                    let amount_of_ticks_passed =
                        (Instant::now() - state.last_tick_time).as_millis() as f32 / 50.;
                    if amount_of_ticks_passed >= 1. {
                        //dbg!(amount_of_ticks_passed);
                        state.tick_update(amount_of_ticks_passed);
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
    chunks: Arc<Mutex<HashMap<(i32, i32), Chunk>>>,
    chunk_offsets: Arc<Mutex<HashMap<(i32, i32), u64>>>,
    previous_chunk: (i32, i32),
    current_chunk_buffer: wgpu::Buffer,
    #[allow(unused)]
    current_chunk_bind_group: wgpu::BindGroup,
    #[allow(unused)]
    current_chunk_bind_group_layout: wgpu::BindGroupLayout,
    seed: u32,
    last_render_time: Instant,
    last_tick_time: Instant,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    diffuse_texture: texture::Texture,
    depth_texture: texture::RawTexture,

    instances: Arc<Mutex<Vec<Instance>>>,
    instance_buffers: Arc<Vec<wgpu::Buffer>>,
    active_instance_buffer: Arc<Mutex<u8>>,
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
            crate::movement::PlayerController::new([16., 150., 16.], &config, &device);
        player_controller
            .camera_uniform
            .update_view_proj(&player_controller.camera, &player_controller.projection);

        let current_chunk = (
            player_controller.camera.position.x as i32 / 32,
            player_controller.camera.position.z as i32 / 32,
        );
        dbg!(current_chunk);
        let current_chunk_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Current chunk buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[current_chunk.0, current_chunk.1]),
        });
        let current_chunk_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Current chunk bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let current_chunk_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Current chunk bind group"),
            layout: &current_chunk_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: current_chunk_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &diffuse_texture.bind_group_layout,
                    &player_controller.camera.bind_group_layout,
                    &current_chunk_bind_group_layout,
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
                //cull_mode: None,
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

        let chunks: Arc<Mutex<HashMap<(i32, i32), Chunk>>> = Arc::new(Mutex::new(HashMap::new()));
        let chunk_offsets: Arc<Mutex<HashMap<(i32, i32), u64>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut handles = Vec::new();
        let offset = Arc::new(Mutex::new(0));
        println!("Generating terrain...");
        for x in -(RENDER_DISTANCE as i32) + 1..RENDER_DISTANCE as i32 {
            for y in -(RENDER_DISTANCE as i32) + 1..RENDER_DISTANCE as i32 {
                let chunks = chunks.clone();
                let chunk_offsets = chunk_offsets.clone();
                let offset = offset.clone();
                let handle = std::thread::spawn(move || {
                    let chunk = Chunk::new((x, y), seed);
                    let position = chunk.position;
                    let mut offset = offset.lock().unwrap();
                    chunks.lock().unwrap().insert(position, chunk);
                    chunk_offsets.lock().unwrap().insert(position, *offset);
                    *offset += CHUNK_BUFFER_SIZE;
                });
                handles.push(handle);
            }
        }

        for handle in handles {
            handle.join().unwrap();
        }
        println!("Done");

        let instances: Vec<Instance> = chunks
            .lock()
            .unwrap()
            .values()
            .flat_map(|chunk| chunk.blocks.iter().map(|block| block.as_instances()))
            .flatten()
            .collect();

        let buffer_size = (RENDER_DISTANCE as u64 * 2 - 1).pow(2) * CHUNK_BUFFER_SIZE;
        let instance_buffer_0 = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            mapped_at_creation: false,
            size: buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let instance_buffer_1 = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            mapped_at_creation: false,
            size: buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        println!("Copying to GPU...");
        for (position, offset) in chunk_offsets.lock().unwrap().iter() {
            let chunks = chunks.lock().unwrap();
            let chunk = chunks.get(position).unwrap();
            let instances = chunk
                .blocks
                .iter()
                .flat_map(Block::as_instances)
                .map(|instance| instance.as_raw())
                .collect::<Vec<_>>();

            queue.write_buffer(
                &instance_buffer_0,
                *offset,
                bytemuck::cast_slice(&instances),
            );
            queue.write_buffer(
                &instance_buffer_1,
                *offset,
                bytemuck::cast_slice(&instances),
            );
        }

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
            chunks,
            chunk_offsets,
            previous_chunk: (0, 0),
            current_chunk_buffer,
            current_chunk_bind_group,
            current_chunk_bind_group_layout,
            seed,
            last_render_time: Instant::now(),
            last_tick_time: Instant::now(),
            instances,
            instance_buffers: vec![instance_buffer_0, instance_buffer_1].into(),
            active_instance_buffer: Arc::new(Mutex::new(0)),
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
                    let cursor_grabbed = self
                        .window
                        .set_cursor_grab(winit::window::CursorGrabMode::Locked);
                    if cursor_grabbed.is_ok() {
                        self.window.set_cursor_visible(false);
                    } else {
                        println!(
                            "Couldn't grab the cursor: {}",
                            cursor_grabbed.err().unwrap()
                        );
                    }
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
        let chunks = self.chunks.clone();
        let chunk_offsets = self.chunk_offsets.clone();
        let instances = self.instances.clone();
        let queue = self.queue.clone();
        let instance_buffers = self.instance_buffers.clone();
        let active_buffer = self.active_instance_buffer.clone();

        self.previous_chunk = current_chunk;
        self.queue.write_buffer(
            &self.current_chunk_buffer,
            0,
            bytemuck::cast_slice(&[current_chunk.0, current_chunk.1]),
        );

        std::thread::spawn(move || {
            State::generate_new_chunks(
                current_chunk,
                previous_chunk,
                seed,
                chunks,
                chunk_offsets,
                instances,
                queue,
                instance_buffers,
                active_buffer,
            )
        });
    }

    fn generate_new_chunks(
        current_chunk: (i32, i32),
        previous_chunk: (i32, i32),
        seed: u32,
        chunks: Arc<Mutex<HashMap<(i32, i32), Chunk>>>,
        chunk_offsets: Arc<Mutex<HashMap<(i32, i32), u64>>>,
        instances: Arc<Mutex<Vec<Instance>>>,
        queue: Queue,
        buffers: Arc<Vec<wgpu::Buffer>>,
        active_buffer: Arc<Mutex<u8>>,
    ) {
        let mut chunks = chunks.lock().unwrap();
        let mut chunk_changes: HashMap<(i32, i32), (i32, i32)> =
            HashMap::with_capacity((RENDER_DISTANCE * 2 - 1) as usize);

        let chunk_position_delta = (
            current_chunk.0 - previous_chunk.0,
            current_chunk.1 - previous_chunk.1,
        );

        if chunk_position_delta.0 != 0 {
            for z in 0..RENDER_DISTANCE * 2 - 1 {
                let new_chunk = (
                    previous_chunk.0 + chunk_position_delta.0 * RENDER_DISTANCE as i32,
                    previous_chunk.1 + z as i32 - RENDER_DISTANCE as i32 + 1,
                );
                let old_chunk = (
                    current_chunk.0 - (RENDER_DISTANCE as i32) * chunk_position_delta.0,
                    new_chunk.1,
                );
                chunk_changes.insert(new_chunk, old_chunk);
            }
        } else if chunk_position_delta.1 != 0 {
            for x in 0..RENDER_DISTANCE * 2 - 1 {
                let new_chunk = (
                    previous_chunk.0 + x as i32 - RENDER_DISTANCE as i32 + 1,
                    previous_chunk.1 + chunk_position_delta.1 * RENDER_DISTANCE as i32,
                );
                let old_chunk = (
                    new_chunk.0,
                    current_chunk.1 - (RENDER_DISTANCE as i32) * chunk_position_delta.1,
                );
                chunk_changes.insert(new_chunk, old_chunk);
            }
        }

        dbg!(&chunk_changes);

        let mut active_buffer = active_buffer.lock().unwrap();
        let buffer_to_write = match *active_buffer {
            0 => 1,
            _ => 0,
        };

        println!("Generating new chunks...");
        for (new_chunk, old_chunk) in chunk_changes {
            dbg!(&chunk_position_delta, &new_chunk, &old_chunk);
            let chunk = Chunk::new(new_chunk, seed);
            let instances: Vec<InstanceRaw> = chunk
                .blocks
                .iter()
                .flat_map(|block| {
                    block
                        .as_instances()
                        .iter()
                        .map(Instance::as_raw)
                        .collect::<Vec<_>>()
                })
                .collect();
            let mut chunk_offsets = chunk_offsets.lock().unwrap();
            let offset = *chunk_offsets.get(&old_chunk).unwrap();

            println!(
                "Chunk: {}, {} -> offset: {}",
                new_chunk.0, new_chunk.1, offset
            );

            let new_size = chunk
                .blocks
                .iter()
                .flat_map(Block::as_instances)
                .collect::<Vec<_>>()
                .len() as u64
                * std::mem::size_of::<InstanceRaw>() as u64;

            let empty_slice = bytemuck::zeroed_slice_box(CHUNK_BUFFER_SIZE as usize);

            // Remove old chunk
            queue.write_buffer(&buffers[buffer_to_write], offset, &empty_slice);
            // Write new chunk if it doesn't override other things
            if new_size <= CHUNK_BUFFER_SIZE {
                queue.write_buffer(
                    &buffers[buffer_to_write],
                    offset,
                    bytemuck::cast_slice(&instances),
                );
                chunk_offsets.insert(new_chunk, offset);
                chunk_offsets.remove(&old_chunk);
            } else {
                println!("New chunk too large!");
            }
            chunks.remove(&old_chunk);
            chunks.insert(new_chunk, chunk);
        }
        drop(queue);
        *active_buffer = buffer_to_write as u8;

        let new_instances: Vec<Instance> = chunks
            .values()
            .flat_map(|chunk| chunk.blocks.iter().map(|block| block.as_instances()))
            .flatten()
            .collect();
        let mut instances = instances.lock().unwrap();
        *instances = new_instances;
    }

    pub fn update(&mut self, dt: Duration) {
        self.player_controller.controller.update_camera(
            &mut self.player_controller.camera,
            self.chunks
                .lock()
                .unwrap()
                .get(&self.previous_chunk)
                .unwrap(),
            dt,
        );

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

    pub fn tick_update(&mut self, ticks: f32) {
        self.player_controller.controller.tick_update_camera(ticks);
        self.update_terrain();
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
        render_pass.set_bind_group(2, &self.current_chunk_bind_group, &[]);

        let active_buffer = *self.active_instance_buffer.lock().unwrap();
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffers[active_buffer as usize].slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.num_indices, 0, 0..instances.len() as _);
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
