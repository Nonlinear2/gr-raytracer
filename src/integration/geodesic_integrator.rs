use std::borrow::Cow;
use std::sync::mpsc;

use wgpu::util::DeviceExt;

use crate::geometry::manifold::PseudoRiemanian4Manifold;
use crate::geometry::photon::{PackedColorResult, PackedPhoton4, Photon4};
use crate::geometry::surface::PackedGpuObject;
use crate::graphics::camera::World;
use crate::graphics::color::Color;

const WORKGROUP_SIZE: u32 = 64;

pub struct GeodesicIntegrator {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    object_buffer: wgpu::Buffer,
}

impl GeodesicIntegrator {
    pub fn new(world: &World) -> Option<Self> {
        let shader_source = Self::get_shader(&*world.manifold);
        let packed_objects: Vec<PackedGpuObject> = world.objects.iter().filter_map(|obj| obj.as_packed_gpu_object()).collect();
        pollster::block_on(Self::new_async(shader_source, packed_objects)).ok()
    }

    pub fn get_shader(_manifold: &dyn PseudoRiemanian4Manifold) -> String {

        // Concatenate shader

        // let common_source = include_str!("../geometry/common.wgsl");
        // let packed_source = include_str!("../geometry/packed_types.wgsl");
        // let euler_source = include_str!("../integration/euler.wgsl");
        // let manifold_source = manifold.get_shader();

        // [common_source, packed_source, euler_source, &manifold_source].join("\n")
        include_str!("../geometry/schwarzschild.wgsl").to_string()
    }

    async fn new_async(shader_source: String, packed_objects: Vec<PackedGpuObject>) -> Result<Self, String> {

        let adapter = wgpu::Instance::default()
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            }).await.map_err(|_| String::from("no gpu adapter found"))?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("black-hole-simulation-gpu-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                experimental_features: wgpu::ExperimentalFeatures::default(),
            }).await.map_err(|err| format!("failed to create gpu device: {err}"))?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("evolve-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source.into())),
        });

        let object_count = packed_objects.len() as u32;
        let object_data = if object_count == 0 {
            vec![PackedGpuObject {
                kind: 0,
                material_kind: 0,
                _pad0: 0,
                _pad1: 0,
                data0: [0.0; 4],
                material_params: [0.0; 4],
                emission_params: [0.0; 4],
            }]
        } else {
            packed_objects
        };

        let object_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("objects"),
            contents: bytemuck::cast_slice(&object_data),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("evolve-bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("evolve-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("evolve-pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("evolve_rays"),
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(Self { device, queue, pipeline, object_buffer })
    }

    pub fn run_kernel(
        &self,
        rays: Vec<Photon4>,
    ) -> Result<Vec<Color>, String> {
        if rays.is_empty() {
            return Ok(Vec::new());
        }

        let packed_rays: Vec<PackedPhoton4> = rays.iter().copied().map(PackedPhoton4::from).collect();

        let buffer_size = std::mem::size_of::<PackedColorResult>() as u64 * rays.len() as u64;

        let input_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("input"),
            contents: bytemuck::cast_slice(&packed_rays),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("output"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let readback_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("schwarzschild-evolve-bind-group"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.object_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder"),
        });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((packed_rays.len() as u32).div_ceil(WORKGROUP_SIZE), 1, 1);
        }

        encoder.copy_buffer_to_buffer(
            &output_buffer,
            0,
            &readback_buffer,
            0,
            buffer_size
        );

        self.queue.submit(Some(encoder.finish()));

        let slice = readback_buffer.slice(..);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });

        self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        }).unwrap();

        match receiver.recv() {
            Ok(Ok(())) => {}
            Ok(Err(err)) => return Err(format!("GPU readback failed: {err}")),
            Err(_) => return Err(String::from("GPU readback channel closed")),
        }

        let mapped = slice.get_mapped_range();
        let packed_output: &[PackedColorResult] = bytemuck::cast_slice(&mapped);
        let output = packed_output.iter().copied().map(|result| {
            Color::new(
                result.color[0] * 255.0,
                result.color[1] * 255.0,
                result.color[2] * 255.0,
            )
        }).collect();
        drop(mapped);
        readback_buffer.unmap();

        Ok(output)
    }
}
