use std::borrow::Cow;
use std::sync::mpsc;

use wgpu::util::DeviceExt;

use crate::geometry::manifold::PseudoRiemanian4Manifold;
use crate::geometry::photon::{Photon4, StopReason, PackedPhoton4, PackedRayResult};

const WORKGROUP_SIZE: u32 = 64;

pub struct GpuGeodesicIntegrator {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
}

impl GpuGeodesicIntegrator {
    pub fn new(manifold: &dyn PseudoRiemanian4Manifold) -> Option<Self> {
        let shader_source = Self::get_shader(manifold);
        pollster::block_on(Self::new_async(shader_source)).ok()
    }

    pub fn get_shader(manifold: &dyn PseudoRiemanian4Manifold) -> String {

        // Concatenate shader

        // let common_source = include_str!("../geometry/common.wgsl");
        // let packed_source = include_str!("../geometry/packed_types.wgsl");
        // let euler_source = include_str!("../integration/euler.wgsl");
        // let manifold_source = manifold.get_shader();

        // [common_source, packed_source, euler_source, &manifold_source].join("\n")
        include_str!("../geometry/schwarzschild.wgsl").to_string()
    }

    async fn new_async(shader_source: String) -> Result<Self, String> {

        let adapter = wgpu::Instance::default()
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| String::from("no gpu adapter found"))?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("black-hole-simulation-gpu-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                experimental_features: wgpu::ExperimentalFeatures::default(),
            })
            .await
            .map_err(|err| format!("failed to create gpu device: {err}"))?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("evolve-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source.into())),
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

        Ok(Self { device, queue, pipeline })
    }

    pub fn integrate(
        &self,
        rays: Vec<Photon4>,
    ) -> Result<Vec<(Photon4, StopReason)>, String> {
        if rays.is_empty() {
            return Ok(Vec::new());
        }

        let packed_rays: Vec<PackedPhoton4> = rays.iter().copied().map(PackedPhoton4::from).collect();

        let buffer_size = std::mem::size_of::<PackedRayResult>() as u64 * rays.len() as u64;

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
        let packed_output: &[PackedRayResult] = bytemuck::cast_slice(&mapped);
        let output = packed_output.iter().copied().map(
            |result| (result.photon.into(), StopReason::try_from(result.stop_reason).unwrap())
        ).collect();
        drop(mapped);
        readback_buffer.unmap();

        Ok(output)
    }
}
