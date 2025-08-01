#[derive(Debug, Clone)]
pub struct GenerationSetup {
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub state_buffer: wgpu::Buffer,
    pub block_buffer: wgpu::Buffer,
    pub face_buffer: wgpu::Buffer,
    pub staging_buffer: wgpu::Buffer,
}

impl GenerationSetup {
    pub fn new(device: &wgpu::Device) -> Self {
        let state_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("State Buffer"),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            size: std::mem::size_of::<State>() as u64,
            mapped_at_creation: false,
        });
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            size: std::mem::size_of::<State>() as u64,
            mapped_at_creation: false,
        });
        let block_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Block Buffer"),
            mapped_at_creation: false,
            size: 1572864,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });
        let face_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Face Buffer"),
            mapped_at_creation: false,
            size: 1572864,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Block bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Block bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        size: None,
                        offset: 0,
                        buffer: &state_buffer,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        size: None,
                        offset: 0,
                        buffer: &block_buffer,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        size: None,
                        offset: 0,
                        buffer: &face_buffer,
                    }),
                },
            ],
        });

        Self {
            bind_group,
            bind_group_layout,
            state_buffer,
            block_buffer,
            face_buffer,
            staging_buffer,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug, Default)]
pub struct State {
    pub block_amount: u32,
    pub face_amount: u32,
    pub chunk_position: [i32; 2],
    pub gradient_vectors: [[f32; 3]; 8],
    padding: [u32; 16],
}
