use pollster::FutureExt;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

pub mod block;
mod camera;
mod hardware;
mod texture;

// const RENDER_DISTANCE: u32 = 2;
const CHUNK_SIZE: (u32, u32, u32) = (32, 128, 32);

pub struct StateApplication<'a> {
    pub state: Option<State<'a>>,
    pub seed: u32,
    pub last_time: std::time::Instant,
}

impl StateApplication<'_> {
    pub fn new(seed: u32) -> Self {
        StateApplication {
            state: None,
            seed,
            last_time: std::time::Instant::now(),
        }
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
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let state = if let Some(state) = &mut self.state {
            state
        } else {
            return;
        };
        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                if state.mouse_pressed {
                    state.camera.controller.handle_mouse(dx, dy);
                }
            }
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                state.resize(physical_size);
            }
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => state.handle_mouse_button(button, button_state.is_pressed()),
            WindowEvent::MouseWheel { delta, .. } => state.handle_mouse_scroll(&delta),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            WindowEvent::RedrawRequested => {
                let dt = self.last_time.elapsed();
                self.last_time = std::time::Instant::now();
                state.update(dt);
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.inner_size();
                        state.resize(size);
                    }
                    Err(e) => {
                        eprintln!("Unable to render {}", e);
                    }
                }
                state.window.request_redraw();
            }
            _ => {}
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
    terrain_gen_pipeline: wgpu::ComputePipeline,
    depth_texture: texture::Texture,
    depth_sampler: wgpu::Sampler,

    camera: camera::Camera,
    mouse_pressed: bool,

    generation_setup: block::GenerationSetup,
    seed: u32,
    textures_bind_group: wgpu::BindGroup,
    face_amount: u32,
    current_chunk: (i32, i32),
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    async fn new(window: Window, seed: u32) -> State<'a> {
        let window = Arc::new(window);
        let window_clone = window.clone();
        let (device, config, queue, surface) = hardware::init(window_clone).await;

        let stone_diffuse_bytes = include_bytes!("../textures/stone.png");
        let stone_diffuse_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            stone_diffuse_bytes,
            "textures/stone.png",
        )
        .unwrap();

        let dirt_diffuse_bytes = include_bytes!("../textures/dirt.png");
        let dirt_diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, dirt_diffuse_bytes, "textures/dirt.png")
                .unwrap();

        let moss_diffuse_bytes = include_bytes!("../textures/moss_block.png");
        let moss_diffuse_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            moss_diffuse_bytes,
            "textures/moss_block.png",
        )
        .unwrap();

        let (depth_texture, depth_sampler) =
            texture::Texture::create_depth_texture(&device, &config, "Depth texture");

        let generation_setup = block::GenerationSetup::new(&device);

        let (camera, camera_bind_group_layout) =
            camera::Camera::new(cgmath::Deg(60.), 0.1, 100., 10., 2.0, &device);

        let (textures_bind_group, textures_bind_group_layout) = texture::create_bind_groups(
            &device,
            &[
                stone_diffuse_texture.raw_texture,
                dirt_diffuse_texture.raw_texture,
                moss_diffuse_texture.raw_texture,
            ],
        );

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let terrain_gen_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Terrain Generation Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../terrain/generation.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &generation_setup.bind_group_layout,
                    &textures_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
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
                // cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
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

        let terrain_gen_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Terrain gen Pipeline Layout"),
                bind_group_layouts: &[&generation_setup.bind_group_layout],
                push_constant_ranges: &[],
            });

        let terrain_gen_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Terrain gen pipeline"),
                layout: Some(&terrain_gen_pipeline_layout),
                module: &terrain_gen_shader,
                entry_point: Some("gen_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        let chunk = crate::terrain::generate(
            &device,
            &queue,
            &terrain_gen_pipeline,
            &generation_setup,
            (0, 0),
        );
        // queue.write_buffer(&block_buffer, 0, bytemuck::cast_slice(&[[0, 0, 0]]));

        Self {
            surface,
            device,
            queue,
            config,
            window,
            camera,
            mouse_pressed: false,
            render_pipeline,
            terrain_gen_pipeline,
            seed,
            textures_bind_group,
            depth_texture,
            depth_sampler,
            generation_setup,
            face_amount: chunk.face_amount,
            current_chunk: chunk.position,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width <= 0 || new_size.height <= 0 {
            return;
        }

        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.camera
            .resize(new_size.width, new_size.height, &self.queue);
        let (depth_texture, depth_sampler) =
            texture::Texture::create_depth_texture(&self.device, &self.config, "Depth texture");
        self.depth_texture = depth_texture;
        self.depth_sampler = depth_sampler;
    }

    fn handle_key(&mut self, _event_loop: &ActiveEventLoop, key: KeyCode, pressed: bool) {
        if !self.camera.controller.handle_key(key, pressed) {
            match (key, pressed) {
                (KeyCode::Escape, true) => {
                    self.mouse_pressed = false;
                    self.window
                        .set_cursor_grab(CursorGrabMode::None)
                        .expect("Failed to release cursor!");
                    self.window.set_cursor_visible(true);
                }
                _ => {}
            }
        }
    }

    fn handle_mouse_button(&mut self, button: MouseButton, _pressed: bool) {
        match button {
            MouseButton::Left => {
                self.mouse_pressed = true;
                self.window
                    .set_cursor_grab(CursorGrabMode::Locked)
                    .expect("Failed to grab cursor!");
                self.window.set_cursor_visible(false);
            }
            _ => {}
        }
    }

    fn handle_mouse_scroll(&mut self, delta: &MouseScrollDelta) {
        self.camera.controller.handle_scroll(delta);
    }

    fn update(&mut self, dt: std::time::Duration) {
        self.camera.update_camera(dt, &self.queue);
        let rounded_camera_position: (i32, i32) = (
            self.camera.position.x.floor() as i32,
            self.camera.position.z.floor() as i32,
        );

        if rounded_camera_position.0 > ((self.current_chunk.0 + 1) * CHUNK_SIZE.0 as i32) - 1 {
            self.current_chunk.0 += 1;
            crate::terrain::generate(
                &self.device,
                &self.queue,
                &self.terrain_gen_pipeline,
                &self.generation_setup,
                self.current_chunk,
            );
        } else if rounded_camera_position.0 < (self.current_chunk.0) * CHUNK_SIZE.0 as i32 {
            self.current_chunk.0 -= 1;
            crate::terrain::generate(
                &self.device,
                &self.queue,
                &self.terrain_gen_pipeline,
                &self.generation_setup,
                self.current_chunk,
            );
        } else if rounded_camera_position.1 > ((self.current_chunk.1 + 1) * CHUNK_SIZE.2 as i32) - 1
        {
            self.current_chunk.1 += 1;
            crate::terrain::generate(
                &self.device,
                &self.queue,
                &self.terrain_gen_pipeline,
                &self.generation_setup,
                self.current_chunk,
            );
        } else if rounded_camera_position.1 < (self.current_chunk.1) * CHUNK_SIZE.2 as i32 {
            self.current_chunk.1 -= 1;
            crate::terrain::generate(
                &self.device,
                &self.queue,
                &self.terrain_gen_pipeline,
                &self.generation_setup,
                self.current_chunk,
            );
        }
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
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.raw_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
        render_pass.set_bind_group(1, &self.generation_setup.bind_group, &[]);
        render_pass.set_bind_group(2, &self.textures_bind_group, &[]);

        render_pass.draw(0..6, 0..(self.face_amount));
        drop(render_pass);
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
