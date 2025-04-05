use cgmath::{vec3, Deg, Quaternion, Rotation3, Vector3};

use super::{Instance, Vertex};

pub const FACE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [0.0, 1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
];

pub const FACE_INDICES: &[u16] = &[
    0, 1, 2, //
    0, 2, 3,
];

pub struct Block {
    position: Vector3<f32>,
}

impl Block {
    pub fn to_instances(&self) -> [Instance; 6] {
        let face_bottom = Instance {
            position: self.position + cgmath::vec3(0., 0., 0.),
            rotation: Quaternion::from_angle_x(Deg(90.0)),
        };
        let face_top = Instance {
            position: self.position + cgmath::vec3(0.0, 1., 1.),
            rotation: Quaternion::from_angle_x(Deg(270.0)),
        };
        let face_front = Instance {
            position: self.position + cgmath::vec3(0.0, 1.0, 0.0),
            rotation: Quaternion::from_angle_x(Deg(180.0)),
        };
        let face_back = Instance {
            position: self.position + cgmath::vec3(0.0, 0.0, 1.0),
            rotation: Quaternion::from_angle_x(Deg(0.0)),
        };
        let face_left = Instance {
            position: self.position + cgmath::vec3(1.0, 0.0, 1.0),
            rotation: Quaternion::from_angle_y(Deg(90.0)),
        };
        let face_right = Instance {
            position: self.position + cgmath::vec3(0.0, 0.0, 0.0),
            rotation: Quaternion::from_angle_y(Deg(270.0)),
        };

        [
            face_bottom,
            face_top,
            face_front,
            face_back,
            face_left,
            face_right,
        ]
    }

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Block {
            position: vec3(x, y, z),
        }
    }

    pub fn plane(position: (f32, f32, f32), width: u32, heigth: u32) -> Vec<Block> {
        let mut blocks = Vec::new();
        for x in 0..width {
            for z in 0..heigth {
                let new_x = position.0 + x as f32;
                let new_z = position.0 + z as f32;
                blocks.push(Block::new(new_x, position.1, new_z));
            }
        }
        blocks
    }
}
