use std::borrow::Cow;
use std::sync::mpsc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::geometry::manifold::{Chart, PseudoRiemanian4Manifold};
use crate::geometry::photon::{Photon4, StopReason};
use crate::geometry::vector::{FourVector, TangentSpace};

const WORKGROUP_SIZE: u32 = 64;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PackedPhoton4 {
    pos: [f32; 4],
    pos_chart: u32,
    pos_padding: [u32; 3],
    vel: [f32; 4],
    vel_space: u32,
    vel_padding: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PackedRayResult {
    photon: PackedPhoton4,
    stop_reason: u32,
    padding: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct StepParams {
    data0: f32,
}

pub struct GpuRayResult {
    pub photon: Photon4,
    pub stop_reason: StopReason,
}

fn chart_to_u32(chart: Chart) -> u32 {
    match chart {
        Chart::CartesianWorld => 0,
        Chart::Cartesian => 1,
        Chart::SphericalZ => 2,
        Chart::SphericalX => 3,
    }
}

fn chart_from_u32(value: u32) -> Chart {
    match value {
        0 => Chart::CartesianWorld,
        1 => Chart::Cartesian,
        2 => Chart::SphericalZ,
        3 => Chart::SphericalX,
        _ => panic!("invalid chart id {value}"),
    }
}

fn tangent_space_to_u32(space: TangentSpace) -> u32 {
    match space {
        TangentSpace::Cartesian => 0,
        TangentSpace::CartesianWorld => 1,
        TangentSpace::SphericalZ => 2,
        TangentSpace::SphericalX => 3,
    }
}

fn tangent_space_from_u32(value: u32) -> TangentSpace {
    match value {
        0 => TangentSpace::Cartesian,
        1 => TangentSpace::CartesianWorld,
        2 => TangentSpace::SphericalZ,
        3 => TangentSpace::SphericalX,
        _ => panic!("invalid tangent space id {value}"),
    }
}

fn stop_reason_from_u32(value: u32) -> StopReason {
    match value {
        0 => StopReason::MaxStepsReached,
        1 => StopReason::BackgroundReached,
        2 => StopReason::ObjectHit,
        3 => StopReason::HorizonHit,
        _ => panic!("invalid stop reason id {value}"),
    }
}

impl From<Photon4> for PackedPhoton4 {
    fn from(photon: Photon4) -> Self {
        Self {
            pos: [photon.pos.t(), photon.pos.r(), photon.pos.theta(), photon.pos.phi()],
            pos_chart: chart_to_u32(photon.pos.chart),
            pos_padding: [0; 3],
            vel: [photon.vel.t(), photon.vel.r(), photon.vel.theta(), photon.vel.phi()],
            vel_space: tangent_space_to_u32(photon.vel.vector_space),
            vel_padding: [0; 3],
        }
    }
}

impl From<PackedPhoton4> for Photon4 {
    fn from(photon: PackedPhoton4) -> Self {
        Photon4::new(
            crate::geometry::point::Point4::new(
                photon.pos[0],
                photon.pos[1],
                photon.pos[2],
                photon.pos[3],
                chart_from_u32(photon.pos_chart),
            ),
            FourVector::new(
                photon.vel[0],
                photon.vel[1],
                photon.vel[2],
                photon.vel[3],
                tangent_space_from_u32(photon.vel_space),
            ),
        )
    }
}

impl From<PackedRayResult> for GpuRayResult {
    fn from(value: PackedRayResult) -> Self {
        Self {
            photon: value.photon.into(),
            stop_reason: stop_reason_from_u32(value.stop_reason),
        }
    }
}

pub struct GpuGeodesicIntegrator {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
}

impl GpuGeodesicIntegrator {
    pub fn new(manifold: &dyn PseudoRiemanian4Manifold) -> Option<Self> {
        let shader_source = manifold.get_shader();
        pollster::block_on(Self::new_async(shader_source)).ok()
    }

    async fn new_async(manifold_shader_source: String) -> Result<Self, String> {
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| String::from("no suitable GPU adapter found"))?;

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
            .map_err(|err| format!("failed to create GPU device: {err}"))?;

        // Concatenate common WGSL and manifold-specific WGSL so files can be modular.
        let common_source = include_str!("../geometry/common.wgsl");
        let manifold_source = &manifold_shader_source;
        let shader_source = [common_source, manifold_source].join("\n");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("schwarzschild-evolve-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source.into())),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("schwarzschild-evolve-bind-group-layout"),
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
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("schwarzschild-evolve-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("schwarzschild-evolve-pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("evolve_schwarzschild_rays"),
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(Self { device, queue, pipeline })
    }

    pub fn evolve_batch(
        &self,
        rays: &[Photon4],
    ) -> Result<Vec<GpuRayResult>, String> {
        if rays.is_empty() {
            return Ok(Vec::new());
        }

        let packed_rays: Vec<PackedPhoton4> = rays.iter().copied().map(PackedPhoton4::from).collect();
        let buffer_size = std::mem::size_of::<PackedRayResult>() as u64 * packed_rays.len() as u64;

        let input_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("schwarzschild-evolve-input"),
            contents: bytemuck::cast_slice(&packed_rays),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("schwarzschild-evolve-output"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let readback_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("schwarzschild-evolve-readback"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let params = StepParams {
            data0: rays.len() as f32,
        };

        let params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("schwarzschild-evolve-params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
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
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("schwarzschild-evolve-encoder"),
        });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("schwarzschild-evolve-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((packed_rays.len() as u32).div_ceil(WORKGROUP_SIZE), 1, 1);
        }

        encoder.copy_buffer_to_buffer(&output_buffer, 0, &readback_buffer, 0, buffer_size);
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
        let output = packed_output.iter().copied().map(GpuRayResult::from).collect();
        drop(mapped);
        readback_buffer.unmap();

        Ok(output)
    }
}
