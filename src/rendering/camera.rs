use cgmath::*;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalPosition;
use winit::event::*;
use winit::keyboard::KeyCode;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera {
    position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    camera_uniform: CameraUniform,
    projection: Projection,
    pub controller: CameraController,
    buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new<F: Into<Rad<f32>>>(
        fovy: F,
        znear: f32,
        zfar: f32,
        speed: f32,
        sensitivity: f32,
        device: &wgpu::Device,
    ) -> (Self, wgpu::BindGroupLayout) {
        let camera_uniform = CameraUniform::new();
        let projection = Projection::new(1920, 1080, fovy, znear, zfar);
        let controller = CameraController::new(speed, sensitivity);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        (
            Self {
                position: point3(0., 0., 0.),
                yaw: Rad(0.),
                pitch: Rad(0.),
                buffer,
                bind_group,
                camera_uniform,
                controller,
                projection,
            },
            bind_group_layout,
        )
    }

    fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }

    pub fn update_view_proj(&mut self, queue: &wgpu::Queue) {
        self.camera_uniform
            .update_view_proj(&self.position, &self.projection, self.calc_matrix());

        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn resize(&mut self, width: u32, height: u32, queue: &wgpu::Queue) {
        self.projection.resize(width, height);
        self.update_view_proj(queue);
    }

    pub fn update_camera(&mut self, dt: std::time::Duration, queue: &wgpu::Queue) {
        self.controller
            .update_camera(&mut self.position, &mut self.pitch, &mut self.yaw, dt);
        self.update_view_proj(queue);
    }
}

#[derive(Debug)]
pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, pressed: bool) -> bool {
        let amount = if pressed { 1.0 } else { 0.0 };
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.amount_forward = amount;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.amount_backward = amount;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.amount_left = amount;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.amount_right = amount;
                true
            }
            KeyCode::Space => {
                self.amount_up = amount;
                true
            }
            KeyCode::ShiftLeft => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn handle_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn handle_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 30.,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
                *scroll as f32 * 60.
            }
        };
    }

    pub fn update_camera(
        &mut self,
        camera_position: &mut Point3<f32>,
        camera_pitch: &mut Rad<f32>,
        camera_yaw: &mut Rad<f32>,
        dt: Duration,
    ) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera_yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        *camera_position +=
            forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        *camera_position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = camera_pitch.0.sin_cos();
        let scrollward =
            Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        *camera_position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera_position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        *camera_yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        *camera_pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if *camera_pitch < -Rad(SAFE_FRAC_PI_2) {
            *camera_pitch = -Rad(SAFE_FRAC_PI_2);
        } else if *camera_pitch > Rad(SAFE_FRAC_PI_2) {
            *camera_pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(
        &mut self,
        position: &Point3<f32>,
        projection: &Projection,
        camera_matrix: Matrix4<f32>,
    ) {
        self.view_position = position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera_matrix).into()
    }
}
