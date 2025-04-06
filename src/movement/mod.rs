use crate::rendering::block::Block;
use crate::rendering::camera::{Camera, CameraUniform, Projection};
use cgmath::{Deg, InnerSpace, Rad, Vector3};
use std::collections::HashSet;
use std::time::Duration;
use winit::event::ElementState;
use winit::keyboard::KeyCode;
const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;
const PLAYER_ACCELERATION: f32 = DRAG;
const BLOCK_FRICTION: f32 = 0.546;
const DRAG: f32 = 0.98;
const GRAVITY: f32 = 0.08;

#[derive(Debug)]
pub struct PlayerController {
    pub camera: Camera,
    pub projection: Projection,
    pub controller: CameraController,
    pub camera_uniform: CameraUniform,
}

impl PlayerController {
    pub fn new(
        position: [f32; 3],
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
    ) -> Self {
        let camera_uniform = CameraUniform::new();
        let camera = Camera::new(position, Rad(0.), Rad(90.), device, camera_uniform);
        let projection = Projection::new(config.width, config.height, Deg(45.), 0.1, 100.);
        let controller = CameraController::new(50.);

        Self {
            camera,
            projection,
            camera_uniform,
            controller,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CameraController {
    input: KeyboardInput,
    movement: (f32, f32, f32),
    velocity: (f32, f32, f32),
    on_ground: bool,
    rotate: (f32, f32),
    sensitivity: f32,
}

#[derive(Debug, Clone)]
struct KeyboardInput {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub sprint: bool,
    pub sneak: bool,
}

impl CameraController {
    pub fn new(sensitivity: f32) -> Self {
        Self {
            input: KeyboardInput {
                x: 0.,
                y: 0.,
                z: 0.,
                sprint: false,
                sneak: false,
            },
            movement: (0., 0., 0.),
            velocity: (0., 0., 0.),
            on_ground: true,
            rotate: (0., 0.),
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            if self.input.sneak {
                0.3
            } else if self.input.sprint {
                1.3
            } else {
                1.0
            }
        } else {
            0.0
        };
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => self.input.z = amount,
            KeyCode::KeyS | KeyCode::ArrowDown => self.input.z = -amount,
            KeyCode::KeyA | KeyCode::ArrowLeft => self.input.x = -amount,
            KeyCode::KeyD | KeyCode::ArrowRight => self.input.x = amount,
            KeyCode::Space => self.input.y = amount,
            KeyCode::ShiftLeft => {
                // self.input.y = -amount;
                self.input.sneak = state.is_pressed()
            }
            KeyCode::ControlLeft => self.input.sprint = state.is_pressed(),
            _ => return false,
        };

        return true;
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate = (mouse_dx as f32, mouse_dy as f32);
    }

    pub fn update_camera(&mut self, camera: &mut Camera, blocks: &HashSet<Block>, dt: Duration) {
        let dt = dt.as_secs_f32();
        self.velocity.0 = ((self.velocity.0 * BLOCK_FRICTION * 0.91)
            + (PLAYER_ACCELERATION * self.input.x * DRAG * (0.6 / BLOCK_FRICTION).powi(3)))
            * dt
            * 20.;

        if self.on_ground {
            self.velocity.1 = 0.;
        } else {
            self.velocity.1 = (self.velocity.1 * GRAVITY.powf(dt * 20.))
                - (GRAVITY * (1. - (GRAVITY.powf(dt * 20.))) / (1. - GRAVITY) * GRAVITY);
        };

        self.velocity.2 = ((self.velocity.2 * BLOCK_FRICTION * 0.91)
            + (PLAYER_ACCELERATION * self.input.z * DRAG * (0.6 / BLOCK_FRICTION).powi(3)))
            * dt
            * 20.;

        self.movement.0 += self.velocity.0;
        self.movement.1 += self.velocity.1;
        self.movement.2 += self.velocity.2;

        // dbg!(&camera.position);

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward * (self.movement.2);
        camera.position += right * (self.movement.0);

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.y += self.movement.1;

        // dbg!(self.movement);
        // dbg!(camera.position);

        // Rotate
        camera.yaw += Rad(self.rotate.0) * self.sensitivity * dt;
        camera.pitch += Rad(-self.rotate.1) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non-cardinal direction.
        self.rotate = (0., 0.);
        self.movement = (0., 0., 0.);

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }

        let block_below = Block::new(
            camera.position.x as i32,
            camera.position.y as i32 - 2,
            camera.position.z as i32,
        );
        self.on_ground = blocks.contains(&block_below);
    }
}
