use crate::rendering::camera::{Camera, CameraUniform, Projection};
use crate::terrain::chunk::Chunk;
use cgmath::{Deg, InnerSpace, Rad, Vector3};
use std::time::Duration;
use winit::event::ElementState;
use winit::keyboard::KeyCode;
const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;
const PLAYER_ACCELERATION: f32 = 3.;
const SLIPPERINESS: f32 = 0.546;
const GRAVITY: f32 = 0.08;
const JUMP_VELOCITY: f32 = 0.42;
const JUMP_COOLDOWN: Duration = Duration::from_millis(500);
// const SECONDS_IN_TICK: f32 = 0.05;
const MOVEMENT_SCALING: f32 = 10.;

mod collision;

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
        let camera = Camera::new(position, Rad(0.), Rad(-90.), device, camera_uniform);
        let projection = Projection::new(config.width, config.height, Deg(90.), 0.1, 100.);
        let controller = CameraController::new(0.01);

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
    pub velocity: (f32, f32, f32),
    on_ground: bool,
    gravity_enabled: bool,
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
    pub last_jump_time: std::time::Instant,
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
                last_jump_time: std::time::Instant::now(),
            },
            movement: (0., 0., 0.),
            velocity: (0., 0., 0.),
            on_ground: true,
            gravity_enabled: false,
            rotate: (0., 0.),
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.
        } else {
            0.
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
            KeyCode::KeyG => {
                if state.is_pressed() {
                    self.gravity_enabled = !self.gravity_enabled
                }
            }
            _ => return false,
        };

        true
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate = (mouse_dx as f32, mouse_dy as f32);
    }

    pub fn tick_update_camera(&mut self, dt: f32) {
        let dt = dt.div_euclid(20.) as u32;
        let dt = if dt < 1 { 1 } else { dt };

        let now = std::time::Instant::now();
        let mut velocity_x = self.velocity.0;
        let mut velocity_y = self.velocity.1;
        let mut velocity_z = self.velocity.2;

        if !self.gravity_enabled {
            self.on_ground = true
        }
        // Check if we can jump
        let do_jump = self.input.y == 1.
            && self.on_ground
            && std::time::Instant::now() - self.input.last_jump_time >= JUMP_COOLDOWN;

        // Sprinting/Sneaking
        let acceleration = if self.input.sprint {
            if self.gravity_enabled {
                PLAYER_ACCELERATION * 1.3
            } else {
                PLAYER_ACCELERATION * 5.
            }
        } else if self.input.sneak && self.gravity_enabled {
            PLAYER_ACCELERATION * 0.3
        } else {
            PLAYER_ACCELERATION
        };

        // Multiply acceleration by 0.2 and set drag to 0.91 when in the air
        let slipperiness = if self.on_ground { SLIPPERINESS } else { 1. };

        // Calculate x velocity
        velocity_x = if self.on_ground {
            velocity_x * slipperiness * 0.91
                + 0.1 * acceleration * self.input.x * (0.6 / slipperiness).powi(3)
        } else {
            velocity_x * slipperiness * 0.91 + 0.02 * acceleration * self.input.x
        };

        // Calculate z velocity
        velocity_z = if self.on_ground {
            velocity_z * slipperiness * 0.91
                + 0.1 * acceleration * self.input.z * (0.6 / slipperiness).powi(3)
        } else {
            velocity_z * slipperiness * 0.91 + 0.02 * acceleration * self.input.z
        };

        // Set vertical velocity
        velocity_y = if do_jump {
            self.input.last_jump_time = now;
            JUMP_VELOCITY
        } else if self.on_ground {
            0.
        } else if now - self.input.last_jump_time >= Duration::from_millis(200) {
            (velocity_y - GRAVITY * dt as f32) * 0.98_f32.powi(dt as i32)
        } else {
            println!("No gravity applied");
            velocity_y
        };

        if !self.gravity_enabled && self.input.y > 0. {
            velocity_y = 2.
        }
        if !self.gravity_enabled && self.input.sneak {
            velocity_y = -2.
        }

        if do_jump && self.input.sprint {
            velocity_x += 0.2 * self.input.x;
            velocity_z += 0.2 * self.input.z;
        }

        self.velocity.0 = velocity_x;
        self.velocity.1 = velocity_y;
        self.velocity.2 = velocity_z;
    }

    pub fn update_camera(&mut self, camera: &mut Camera, chunk: &Chunk, dt: Duration) {
        let dt = dt.as_secs_f32();
        let old_position = camera.position;

        // Add velocity to position
        self.movement.0 += self.velocity.0 * MOVEMENT_SCALING;
        self.movement.1 += self.velocity.1 * MOVEMENT_SCALING;
        self.movement.2 += self.velocity.2 * MOVEMENT_SCALING;

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0., yaw_sin).normalize();
        let up = Vector3::new(0., 1., 0.).normalize();
        let right = Vector3::new(-yaw_sin, 0., yaw_cos).normalize();
        let world_movement =
            forward * self.movement.2 + up * self.movement.1 + right * self.movement.0;

        let collisions =
            collision::check_player_block(camera.position, world_movement, &chunk.blocks);

        if !collisions.0 {
            camera.position.x += world_movement.x * dt
        }
        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        self.on_ground = collisions.1 && world_movement.y <= 0.;
        if self.velocity.1.abs() > 0.005 && !self.on_ground {
            camera.position.y += world_movement.y * dt
        };
        if !collisions.2 {
            camera.position.z += world_movement.z * dt
        }

        // println!(
        //     "y: {:?}, vy: {:?}, blocks/s: {:?}, my: {:?}",
        //     camera.position.y,
        //     self.velocity.1,
        //     ((camera.position.y - old_position.y) / dt),
        //     self.movement.1
        // );

        // Rotate
        camera.yaw += Rad(self.rotate.0) * self.sensitivity;
        camera.pitch += Rad(-self.rotate.1) * self.sensitivity;
        //dbg!(camera.yaw);

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
    }
}
