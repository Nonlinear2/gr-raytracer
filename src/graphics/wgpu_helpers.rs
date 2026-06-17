pub struct Buffers {
    pub input: wgpu::Buffer,
    pub output: wgpu::Buffer,
    pub objects: wgpu::Buffer,
    pub readback: wgpu::Buffer,
    pub trace_output: wgpu::Buffer,
    pub trace_readback: wgpu::Buffer,
}

pub enum BindEntry {
    StorageBuffer { binding: u32, read_only: bool },
    Texture { binding: u32 },
    Sampler { binding: u32 },
}

impl BindEntry {
    pub fn build(self) -> wgpu::BindGroupLayoutEntry {
        match self {
            BindEntry::StorageBuffer { binding, read_only } => {
                wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            }
            BindEntry::Texture { binding } => {
                wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }
            }
            BindEntry::Sampler { binding } => {
                wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(
                        wgpu::SamplerBindingType::Filtering,
                    ),
                    count: None,
                }
            }
        }
    }
}