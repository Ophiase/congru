use bytemuck::{Pod, Zeroable};
use congru::DistanceField;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GridUniform {
    voxel: f32,
    width: u32,
    height: u32,
    depth: u32,
}

pub(super) fn uniform_of(device: &wgpu::Device, field: &DistanceField) -> Option<wgpu::Buffer> {
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

pub(super) fn pipeline_of(device: &wgpu::Device) -> wgpu::ComputePipeline {
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

#[expect(
    clippy::cast_possible_truncation,
    reason = "a place in the grid in single precision, far finer than a voxel"
)]
pub(super) const fn narrowed(value: f64) -> f32 {
    value as f32
}
