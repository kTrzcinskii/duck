use std::mem;

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::bind_groups::RendererBindGroupsLayout;

pub struct CubemapPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl CubemapPipeline {
    pub fn new(
        device: &wgpu::Device,
        global_layout: &wgpu::BindGroupLayout,
        water_render_layout: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/cubemap.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Skybox Pipeline Layout"),
            bind_group_layouts: &[Some(global_layout), Some(water_render_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Skybox Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[CubemapVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                front_face: wgpu::FrontFace::Ccw,
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
        CubemapPipeline { pipeline }
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}

pub struct Cubemap {
    pipeline: CubemapPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Cubemap {
    const VERTICES: &[CubemapVertex] = &[
        CubemapVertex {
            position: [-1.0, -1.0, -1.0],
        },
        CubemapVertex {
            position: [1.0, -1.0, -1.0],
        },
        CubemapVertex {
            position: [1.0, 1.0, -1.0],
        },
        CubemapVertex {
            position: [-1.0, 1.0, -1.0],
        },
        CubemapVertex {
            position: [-1.0, -1.0, 1.0],
        },
        CubemapVertex {
            position: [1.0, -1.0, 1.0],
        },
        CubemapVertex {
            position: [1.0, 1.0, 1.0],
        },
        CubemapVertex {
            position: [-1.0, 1.0, 1.0],
        },
    ];

    const INDICES: &[u16] = &[
        0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 3, 7, 0, 7, 4, 1, 5, 6, 1, 6, 2, 0, 4, 5, 0, 5, 1,
        3, 2, 6, 3, 6, 7,
    ];

    pub fn new(
        device: &wgpu::Device,
        layouts: &RendererBindGroupsLayout,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let pipeline =
            CubemapPipeline::new(device, layouts.global(), layouts.water(), surface_format);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cubemap Vertex Buffer"),
            contents: bytemuck::cast_slice(Self::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cubemap Index Buffer"),
            contents: bytemuck::cast_slice(Self::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        Cubemap {
            pipeline,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        self.pipeline.pipeline()
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    pub fn indices_len(&self) -> u32 {
        Self::INDICES.len() as _
    }
}

pub fn load_cubemap(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<wgpu::Texture> {
    let image_bytes = [
        include_bytes!("../assets/textures/cubemap/px.png").as_slice(),
        include_bytes!("../assets/textures/cubemap/nx.png").as_slice(),
        include_bytes!("../assets/textures/cubemap/py.png").as_slice(),
        include_bytes!("../assets/textures/cubemap/ny.png").as_slice(),
        include_bytes!("../assets/textures/cubemap/pz.png").as_slice(),
        include_bytes!("../assets/textures/cubemap/nz.png").as_slice(),
    ];

    let mut images = Vec::with_capacity(image_bytes.len());
    for bytes in image_bytes {
        let img = image::load_from_memory(bytes)?.to_rgba8();
        images.push(img);
    }

    let size = images[0].width();

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Cubemap"),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 6,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    for (i, img) in images.iter().enumerate() {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: i as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            img,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size),
                rows_per_image: Some(size),
            },
            wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
        );
    }

    Ok(texture)
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CubemapVertex {
    pub position: [f32; 3],
}

impl CubemapVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<CubemapVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}
