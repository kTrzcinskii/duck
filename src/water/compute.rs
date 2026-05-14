use crate::bind_groups::{RendererBindGroupsLayout, WaterComputeBindGroup};

pub struct WaterComputePipeline {
    pipeline: wgpu::ComputePipeline,
}

impl WaterComputePipeline {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/water_compute.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Water Compute Pipeline Layout"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Water Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Self { pipeline }
    }

    pub fn pipeline(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }
}

pub struct WaterSimulation {
    compute_bg: WaterComputeBindGroup,
    pipeline: WaterComputePipeline,
}

impl WaterSimulation {
    pub fn new(device: &wgpu::Device, layouts: &RendererBindGroupsLayout) -> Self {
        let compute_bg = WaterComputeBindGroup::new(device, layouts.water_compute());
        let pipeline = WaterComputePipeline::new(device, layouts.water_compute());
        Self {
            compute_bg,
            pipeline,
        }
    }

    pub fn step(&mut self, encoder: &mut wgpu::CommandEncoder) {
        self.compute_bg.step(encoder, self.pipeline.pipeline());
    }

    pub fn disturb(&self, queue: &wgpu::Queue, col: u32, row: u32, amount: f32) {
        self.compute_bg.disturb(queue, col, row, amount);
    }

    pub fn normal_texture(&self) -> &wgpu::Texture {
        self.compute_bg.normal_texture()
    }
}
