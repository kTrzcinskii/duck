use crate::bind_groups::{RendererBindGroupsLayout, WaterRenderBindGroup};

pub struct WaterRenderPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl WaterRenderPipeline {
    pub fn new(
        device: &wgpu::Device,
        global_layout: &wgpu::BindGroupLayout,
        water_render_layout: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("../../shaders/water.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Water Render Pipeline Layout"),
            bind_group_layouts: &[Some(global_layout), Some(water_render_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Water Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        WaterRenderPipeline { pipeline }
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}

pub struct WaterSurface {
    render_bg: WaterRenderBindGroup,
    pipeline: WaterRenderPipeline,
}

impl WaterSurface {
    pub fn new(
        device: &wgpu::Device,
        layouts: &RendererBindGroupsLayout,
        normal_texture: &wgpu::Texture,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let render_bg = WaterRenderBindGroup::new(device, layouts.water(), normal_texture);
        let pipeline =
            WaterRenderPipeline::new(device, layouts.global(), layouts.water(), surface_format);
        Self {
            render_bg,
            pipeline,
        }
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        self.pipeline.pipeline()
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        self.render_bg.bind_group()
    }
}
