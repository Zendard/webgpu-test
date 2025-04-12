use super::{instance::Face, Instance, Vertex};
use cgmath::Vector3;

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

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct Block {
    pub position: (u8, u8, u8),
    pub visible_faces: u8,
}

impl Block {
    pub fn new(x: u8, y: u8, z: u8) -> Self {
        Block {
            position: (x, y, z),
            visible_faces: 0b000000,
        }
    }

    pub fn as_instances(&self) -> Vec<Instance> {
        let position_vec3: Vector3<u8> = cgmath::Vector3::from(self.position);

        let mut faces = Vec::new();

        if (self.visible_faces & 0b100000) > 0 {
            let face_left = Instance {
                position: position_vec3 + cgmath::vec3(0, 0, 0),
                face: Face::Left,
            };
            faces.push(face_left);
        }
        if (self.visible_faces & 0b010000) > 0 {
            let face_right = Instance {
                position: position_vec3 + cgmath::vec3(1, 0, 1),
                face: Face::Right,
            };
            faces.push(face_right);
        }
        if (self.visible_faces & 0b001000) > 0 {
            let face_bottom = Instance {
                position: position_vec3,
                face: Face::Bottom,
            };
            faces.push(face_bottom);
        }
        if (self.visible_faces & 0b000100) > 0 {
            let face_top = Instance {
                position: position_vec3 + cgmath::vec3(0, 1, 1),
                face: Face::Top,
            };
            faces.push(face_top);
        }
        if (self.visible_faces & 0b000010) > 0 {
            let face_front = Instance {
                position: position_vec3 + cgmath::vec3(0, 1, 0),
                face: Face::Front,
            };
            faces.push(face_front);
        }
        if (self.visible_faces & 0b000001) > 0 {
            let face_back = Instance {
                position: position_vec3 + cgmath::vec3(0, 0, 1),
                face: Face::Back,
            };
            faces.push(face_back);
        }

        faces
    }
}
