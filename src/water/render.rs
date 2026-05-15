use crate::{bind_groups::RendererBindGroupsLayout, cubemap};

use log::error;

pub struct WaterRenderBindGroup {
    bind_group: wgpu::BindGroup,
    _cubemap: wgpu::Texture,
}

impl WaterRenderBindGroup {
    pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Water Render Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        normal_texture: &wgpu::Texture,
        cubemap: wgpu::Texture,
    ) -> Self {
        let normal_view = normal_texture.create_view(&Default::default());

        let normal_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Water Normal Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let cubemap_view = cubemap.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let cubemap_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Water Cubemap Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&normal_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&cubemap_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&cubemap_sampler),
                },
            ],
            label: Some("Water Render Bind Group"),
        });
        WaterRenderBindGroup {
            bind_group,
            _cubemap: cubemap,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

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
        queue: &wgpu::Queue,
        layouts: &RendererBindGroupsLayout,
        normal_texture: &wgpu::Texture,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let cubemap = match cubemap::load_cubemap(device, queue) {
            Ok(c) => c,
            Err(err) => {
                error!("Failed to load cubemap: {err}");
                panic!("{}", err);
            }
        };
        let render_bg = WaterRenderBindGroup::new(device, layouts.water(), normal_texture, cubemap);
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
