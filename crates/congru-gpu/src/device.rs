use std::sync::Mutex;

use bytemuck::{Pod, Zeroable};
use congru::{DistanceField, Point};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

const WORKGROUP: u32 = 256;
const ANSWER_BYTES: u64 = 16;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GridUniform {
    voxel: f32,
    width: u32,
    height: u32,
    depth: u32,
}

/// The device, the field on it and the shader that reads it. One batch at a time.
pub struct Reader {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    values: wgpu::Buffer,
    origin: congru::Vector,
    name: String,
    busy: Mutex<()>,
}

impl std::fmt::Debug for Reader {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Reader")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl Reader {
    /// `None` when no adapter or device answers, or the field is past its limits.
    pub fn new(field: &DistanceField) -> Option<Self> {
        pollster::block_on(Self::started(field))
    }

    async fn started(field: &DistanceField) -> Option<Self> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .ok()?;
        let limits = adapter.limits();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_limits: limits,
                ..Default::default()
            })
            .await
            .ok()?;
        let uniform = uniform_of(&device, field)?;
        let values = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(field.values()),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let pipeline = pipeline_of(&device);
        Some(Self {
            layout: pipeline.get_bind_group_layout(0),
            device,
            queue,
            pipeline,
            uniform,
            values,
            origin: field.grid().origin,
            name: adapter.get_info().name,
            busy: Mutex::new(()),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// For each query, the gradient then the signed distance, a huge one outside.
    pub fn read(&self, queries: &[Point]) -> Vec<[f32; 4]> {
        let Ok(_turn) = self.busy.lock() else {
            return Vec::new();
        };
        let places: Vec<[f32; 4]> = queries
            .iter()
            .map(|query| {
                let place = query.coords - self.origin;
                [narrowed(place.x), narrowed(place.y), narrowed(place.z), 0.0]
            })
            .collect();
        let input = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&places),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let bytes = ANSWER_BYTES * queries.len() as u64;
        let output = self.buffer(
            bytes,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        );
        let staging = self.buffer(
            bytes,
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        );
        self.dispatch(&input, &output, &staging, bytes, queries.len());
        self.mapped(&staging)
    }

    fn buffer(&self, size: u64, usage: wgpu::BufferUsages) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage,
            mapped_at_creation: false,
        })
    }

    fn dispatch(
        &self,
        input: &wgpu::Buffer,
        output: &wgpu::Buffer,
        staging: &wgpu::Buffer,
        bytes: u64,
        count: usize,
    ) {
        let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &[
                bound(0, &self.uniform),
                bound(1, &self.values),
                bound(2, input),
                bound(3, output),
            ],
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &group, &[]);
            let groups = u32::try_from(count).unwrap_or(u32::MAX).div_ceil(WORKGROUP);
            pass.dispatch_workgroups(groups, 1, 1);
        }
        encoder.copy_buffer_to_buffer(output, 0, staging, 0, bytes);
        self.queue.submit([encoder.finish()]);
    }

    fn mapped(&self, staging: &wgpu::Buffer) -> Vec<[f32; 4]> {
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        if self
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .is_err()
        {
            return Vec::new();
        }
        let answers = bytemuck::cast_slice(&slice.get_mapped_range()).to_vec();
        staging.unmap();
        answers
    }
}

fn uniform_of(device: &wgpu::Device, field: &DistanceField) -> Option<wgpu::Buffer> {
    let grid = field.grid();
    Some(device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::bytes_of(&GridUniform {
            voxel: narrowed(grid.voxel),
            width: u32::try_from(grid.sizes[0]).ok()?,
            height: u32::try_from(grid.sizes[1]).ok()?,
            depth: u32::try_from(grid.sizes[2]).ok()?,
        }),
        usage: wgpu::BufferUsages::UNIFORM,
    }))
}

fn pipeline_of(device: &wgpu::Device) -> wgpu::ComputePipeline {
    let module = device.create_shader_module(wgpu::include_wgsl!("read.wgsl"));
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &module,
        entry_point: Some("read"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn bound(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "a place in the grid in single precision, far finer than a voxel"
)]
const fn narrowed(value: f64) -> f32 {
    value as f32
}
