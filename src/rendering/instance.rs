#[derive(Debug, Clone, Copy)]
pub enum Face {
    Left = 0b000,
    Right = 0b001,
    Top = 0b010,
    Bottom = 0b011,
    Front = 0b100,
    Back = 0b101,
}

#[derive(Debug, Clone, Copy)]
pub struct Instance {
    pub position: cgmath::Vector3<u8>,
    pub face: Face,
}

impl Instance {
    pub fn as_raw(&self) -> InstanceRaw {
        let raw_description: u32 = ((self.position.x as u32) << 12)
            + ((self.position.y as u32) << 6)
            + (self.position.z as u32)
            + ((self.face as u32) << 18);
        InstanceRaw(raw_description)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// fffxxxxxxyyyyyyzzzzzz
// f = face         | x = x position in chunk
// 000 -> left      | y = y position in chunk
// 001 -> right     | z = z position in chunk
// 010 -> top
// 011 -> bottom
// 100 -> front
// 101 -> back
pub struct InstanceRaw(u32);
impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 5,
                format: wgpu::VertexFormat::Uint32,
            }],
        }
    }
}
