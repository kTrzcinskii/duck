use std::mem;

use anyhow::{Context, Result, bail};
use bytemuck::{Pod, Zeroable};
use glam::{Quat, Vec3};
use image::ImageReader;
use log::error;
use wgpu::util::DeviceExt;

use crate::{
    bind_groups::RendererBindGroupsLayout,
    model::{Model, ModelBuffer},
};

pub struct DuckBindGroup {
    bind_group: wgpu::BindGroup,
    model_buffer: wgpu::Buffer,
    _texture: wgpu::Texture,
}

impl DuckBindGroup {
    pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Duck Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, queue: &wgpu::Queue) -> Self {
        let model_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Model Buffer"),
            size: size_of::<ModelBuffer>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let texture = match load_texture(device, queue) {
            Ok(t) => t,
            Err(err) => {
                error!("Failed to load duck texture: {err}");
                panic!("{}", err);
            }
        };
        let texture_view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Duck Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: model_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Duck Bind Group"),
        });

        DuckBindGroup {
            bind_group,
            model_buffer,
            _texture: texture,
        }
    }

    pub fn update_buffer(&self, queue: &wgpu::Queue, buffer: ModelBuffer) {
        queue.write_buffer(&self.model_buffer, 0, bytemuck::cast_slice(&[buffer]));
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

pub struct DuckPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl DuckPipeline {
    pub fn new(
        device: &wgpu::Device,
        global_layout: &wgpu::BindGroupLayout,
        duck_layout: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("../../shaders/duck.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Duck Pipeline Layout"),
            bind_group_layouts: &[Some(global_layout), Some(duck_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Duck Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[DuckVertex::desc()],
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
                cull_mode: Some(wgpu::Face::Front),
                front_face: wgpu::FrontFace::Cw,
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

        DuckPipeline { pipeline }
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}

// TODO: currently duck is not properly interacting with the water
pub struct Duck {
    pipeline: DuckPipeline,
    model: Model,
    render_bg: DuckBindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    indices_len: u32,
}

impl Duck {
    pub fn new(
        device: &wgpu::Device,
        layouts: &RendererBindGroupsLayout,
        surface_format: wgpu::TextureFormat,
        queue: &wgpu::Queue,
    ) -> Self {
        let pipeline = DuckPipeline::new(device, layouts.global(), layouts.duck(), surface_format);
        let render_bg = DuckBindGroup::new(device, layouts.duck(), queue);
        let (vertices, indices) = match Self::load_from_file() {
            Ok(data) => data,
            Err(err) => {
                error!("Failed to load duck from file: {err}");
                panic!("{}", err);
            }
        };
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Duck Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Duck Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let model = Model::new(
            Vec3::new(0.0, -0.05, 0.0),
            Quat::IDENTITY,
            Self::model_scale(),
        );
        render_bg.update_buffer(queue, (&model).into());
        Duck {
            pipeline,
            model,
            render_bg,
            vertex_buffer,
            index_buffer,
            indices_len: indices.len() as _,
        }
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        self.pipeline.pipeline()
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        self.render_bg.bind_group()
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    pub fn indices_len(&self) -> u32 {
        self.indices_len
    }

    pub fn update_model(&mut self, queue: &wgpu::Queue, update: impl FnOnce(&mut Model)) {
        update(&mut self.model);
        self.render_bg.update_buffer(queue, (&self.model).into());
    }

    fn load_from_file() -> Result<(Vec<DuckVertex>, Vec<u16>)> {
        const NUMS_PER_VERTEX_LINE: usize = 8;
        const NUMS_PER_TRIANGLE_LINE: usize = 3;

        let content = include_str!("../../assets/meshes/duck.txt");
        let mut lines = content.lines();
        let vertex_count: usize = lines
            .next()
            .context("missing vertex count line")?
            .trim()
            .parse()?;

        let mut vertices = Vec::with_capacity(vertex_count);

        let mut vertex_nums = Vec::with_capacity(NUMS_PER_VERTEX_LINE);

        for _ in 0..vertex_count {
            vertex_nums.clear();
            let line = lines.next().context("missing next vertex line")?;

            let elements = line.split_whitespace();
            for element in elements {
                let flt = element.parse::<f32>().context("failed to parse f32")?;
                vertex_nums.push(flt);
            }

            if vertex_nums.len() != NUMS_PER_VERTEX_LINE {
                bail!(
                    "incorrect data for vertex: got {} floats, expected {}",
                    vertex_nums.len(),
                    NUMS_PER_VERTEX_LINE
                );
            }

            vertices.push(DuckVertex {
                position: [vertex_nums[0], vertex_nums[1], vertex_nums[2]],
                normal: [vertex_nums[3], vertex_nums[4], vertex_nums[5]],
                tex_coords: [vertex_nums[6], vertex_nums[7]],
            });
        }

        let triangle_count: usize = lines
            .next()
            .context("missing triangle count line")?
            .trim()
            .parse()?;

        let mut indices = Vec::with_capacity(triangle_count * 3);

        let mut triangle_nums = Vec::with_capacity(NUMS_PER_TRIANGLE_LINE);

        for _ in 0..triangle_count {
            triangle_nums.clear();
            let line = lines.next().context("missing next triangle line")?;

            let elements = line.split_whitespace();
            for element in elements {
                let n = element.parse::<u16>().context("failed to parse u16")?;
                triangle_nums.push(n);
            }

            if triangle_nums.len() != NUMS_PER_TRIANGLE_LINE {
                bail!(
                    "incorrect data for triangle: got {} u16s, expected {}",
                    triangle_nums.len(),
                    NUMS_PER_TRIANGLE_LINE
                );
            }

            indices.extend_from_slice(&triangle_nums);
        }

        Ok((vertices, indices))
    }

    fn model_scale() -> Vec3 {
        Vec3::ONE / 600.0
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DuckVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl DuckVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<DuckVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

fn load_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<wgpu::Texture> {
    // TODO: try to make this path univeral, (meaning i can run from other directories, baked like with include_str)
    let img = ImageReader::open("assets/textures/ducktex.jpg")?
        .decode()?
        .to_rgba8();

    let (width, height) = img.dimensions();

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Duck Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &img,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    Ok(texture)
}
