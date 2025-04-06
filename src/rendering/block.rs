use std::collections::HashSet;

use cgmath::{Deg, Quaternion, Rotation3, Vector3};

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

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct Block {
    position: (i32, i32, i32),
}

impl Block {
    pub fn to_instances(&self) -> [Instance; 6] {
        let position_f32: (f32, f32, f32) = (
            self.position.0 as f32,
            self.position.1 as f32,
            self.position.2 as f32,
        );
        let position_vec3: Vector3<f32> = cgmath::Vector3::from(position_f32);
        let face_bottom = Instance {
            position: position_vec3,
            rotation: Quaternion::from_angle_x(Deg(90.0)),
        };
        let face_top = Instance {
            position: position_vec3 + cgmath::vec3(0., 1., 1.),
            rotation: Quaternion::from_angle_x(Deg(270.0)),
        };
        let face_front = Instance {
            position: position_vec3 + cgmath::vec3(0., 1., 0.),
            rotation: Quaternion::from_angle_x(Deg(180.0)),
        };
        let face_back = Instance {
            position: position_vec3 + cgmath::vec3(0., 0., 1.),
            rotation: Quaternion::from_angle_x(Deg(0.0)),
        };
        let face_left = Instance {
            position: position_vec3 + cgmath::vec3(1., 0., 1.),
            rotation: Quaternion::from_angle_y(Deg(90.0)),
        };
        let face_right = Instance {
            position: position_vec3 + cgmath::vec3(0., 0., 0.),
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

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Block {
            position: (x, y, z),
        }
    }

    pub fn plane(position: (i32, i32, i32), width: u32, heigth: u32) -> HashSet<Block> {
        let mut blocks = HashSet::new();
        for x in 0..width {
            for z in 0..heigth {
                let new_x = position.0 + x as i32;
                let new_z = position.0 + z as i32;
                blocks.insert(Block::new(new_x, position.1, new_z));
            }
        }
        blocks
    }
}
