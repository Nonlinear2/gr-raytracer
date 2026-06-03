use std::borrow::Cow;
use std::sync::mpsc;

use bytemuck::Zeroable;
use wgpu::util::DeviceExt;

use crate::geometry::manifold::PseudoRiemanian4Manifold;
use crate::geometry::photon::{PackedPhoton4, PackedTraceResult, Photon4};
use crate::graphics::texture::Textures;
use crate::graphics::surface::PackedObject;
use crate::graphics::camera::{Objects, World};
use crate::graphics::color::{Color, PackedColorResult};

const WORKGROUP_SIZE: u32 = 64;

pub struct GeodesicIntegrator {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    object_buffer: wgpu::Buffer,
    texture_buffer: wgpu::Buffer,

    debug_ray_trajectory: bool,
}

impl GeodesicIntegrator {
    pub fn new(world: &World, max_steps: u32, debug_ray_trajectory: bool) -> Option<Self> {
        let shader_source = Self::get_shader(&*world.manifold, max_steps, debug_ray_trajectory);

        pollster::block_on(Self::new_async(shader_source, &world.objects, &world.textures, debug_ray_trajectory)).ok()
    }

    pub fn get_shader(_manifold: &dyn PseudoRiemanian4Manifold, max_steps: u32, debug_ray_trajectory: bool) -> String {

        // Concatenate shader

        // let common_source = include_str!("../geometry/common.wgsl");
        // let packed_source = include_str!("../geometry/packed_types.wgsl");
        // let euler_source = include_str!("../integration/euler.wgsl");
        // let manifold_source = manifold.get_shader();

        // [common_source, packed_source, euler_source, &manifold_source].join("\n")
        include_str!("../geometry/schwarzschild.wgsl")
            .replace(
                "const MAX_STEPS: u32 = 0u;", 
                &format!("const MAX_STEPS: u32 = {};", max_steps)
            ).replace(
                "DEBUG_RAY_TRAJECTORY: bool = false;", 
                &format!("const DEBUG_RAY_TRAJECTORY: bool = {};", debug_ray_trajectory
            )
        )
    }

    async fn new_async(shader_source: String, objects: &Objects, textures: &Textures, debug_ray_trajectory: bool) -> Result<Self, String> {

        let packed_objects: Vec<PackedObject> = objects.iter().filter_map(|obj| obj.as_packed_object()).collect();
        let packed_textures = textures.as_packed_texture();
    
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
        
        let object_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("objects"),
            contents: bytemuck::cast_slice(&packed_objects),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let texture_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("texture"),
            contents: packed_textures.as_slice(),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let bind_group_entries = vec![
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
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ];

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("evolve-bind-group-layout"),
            entries: &bind_group_entries,
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

        Ok(Self { device, queue, pipeline, object_buffer, texture_buffer, debug_ray_trajectory })
    }

    fn create_buffers(&self, packed_rays: &[PackedPhoton4], rays_byte_size: u64) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer, wgpu::Buffer, Option<wgpu::Buffer>) {
        let input_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("input"),
            contents: bytemuck::cast_slice(packed_rays),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("output"),
            size: rays_byte_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let readback_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: rays_byte_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let trace_output_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("trace-output"),
            contents: bytemuck::bytes_of(&PackedTraceResult::zeroed()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
    
        let trace_readback = if self.debug_ray_trajectory {
            Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("trace-readback"),
                size: std::mem::size_of::<PackedTraceResult>() as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }))
        } else {
            None
        };

        (input_buffer, output_buffer, readback_buffer, trace_output_buffer, trace_readback)
    }

    fn create_bind_group(&self, input_buffer: &wgpu::Buffer, output_buffer: &wgpu::Buffer, trace_output_buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        let entries = vec![
            wgpu::BindGroupEntry { binding: 0, resource: input_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: output_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: self.object_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: trace_output_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 4, resource: self.texture_buffer.as_entire_binding() },
        ];

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("schwarzschild-evolve-bind-group"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &entries,
        })
    }

    fn dispatch_and_readback(&self, bind_group: &wgpu::BindGroup, packed_rays_len: usize, output_buffer: &wgpu::Buffer, readback_buffer: &wgpu::Buffer, buffer_size: u64, trace_output_buffer: &wgpu::Buffer, trace_readback: Option<&wgpu::Buffer>) -> (Vec<Color>, Option<Vec<PackedTraceResult>>) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("encoder") });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("pass"), timestamp_writes: None });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.dispatch_workgroups((packed_rays_len as u32).div_ceil(WORKGROUP_SIZE), 1, 1);
        }

        encoder.copy_buffer_to_buffer(output_buffer, 0, readback_buffer, 0, buffer_size);
        let trace_byte_size = std::mem::size_of::<PackedTraceResult>() as u64;
        if let Some(trace_readback) = trace_readback {
            encoder.copy_buffer_to_buffer(trace_output_buffer, 0, trace_readback, 0, trace_byte_size);
        }

        self.queue.submit(Some(encoder.finish()));

        let slice = readback_buffer.slice(..);
        let trace_slice_opt = trace_readback.map(|b| b.slice(..));

        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| { let _ = sender.send(result); });

        let (_trace_sender, trace_receiver) = if let Some(trace_slice) = trace_slice_opt.as_ref() {
            let (s, r) = mpsc::channel();
            let s_clone = s.clone();
            trace_slice.map_async(wgpu::MapMode::Read, move |res| { let _ = s_clone.send(res); });
            (Some(s), Some(r))
        } else {
            (None, None)
        };

        self.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).unwrap();

        match receiver.recv() { Ok(Ok(())) => {}, _ => panic!("failed to map output buffer") }
        if let Some(trace_receiver) = trace_receiver {
            match trace_receiver.recv() { Ok(Ok(())) => {}, _ => panic!("failed to map trace buffer") }
        }

        let mapped = slice.get_mapped_range();
        let packed_output: &[PackedColorResult] = bytemuck::cast_slice(&mapped);
        let output: Vec<Color> = packed_output.iter().copied().map(|result| {
            Color::new(result.color[0] * 255.0, result.color[1] * 255.0, result.color[2] * 255.0)
        }).collect();

        let trace_results = if let Some(trace_slice) = trace_slice_opt {
            let mapped_trace = trace_slice.get_mapped_range();
            let packed_trace: &[PackedTraceResult] = bytemuck::cast_slice(&mapped_trace);
            let traces: Vec<PackedTraceResult> = packed_trace.iter().copied().collect();
            drop(mapped_trace);
            if let Some(tb) = trace_readback {
                tb.unmap();
            }
            Some(traces)
        } else {
            None
        };

        drop(mapped);
        readback_buffer.unmap();

        (output, trace_results)
    }

    pub fn run_kernel(&self, rays: Vec<Photon4>) -> (Vec<Color>, Option<Vec<PackedTraceResult>>) {

        let packed_rays: Vec<PackedPhoton4> = rays.iter().copied().map(PackedPhoton4::from).collect();
        let rays_byte_size = std::mem::size_of::<PackedColorResult>() as u64 * packed_rays.len() as u64;

        let (input_buffer, output_buffer, readback_buffer, trace_output_buffer, trace_readback) = self.create_buffers(&packed_rays, rays_byte_size);
        let bind_group = self.create_bind_group(&input_buffer, &output_buffer, &trace_output_buffer);
        self.dispatch_and_readback(&bind_group, packed_rays.len(), &output_buffer, &readback_buffer, rays_byte_size, &trace_output_buffer, trace_readback.as_ref())
    }
}
