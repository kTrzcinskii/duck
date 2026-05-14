use wgpu::util::DeviceExt;

use crate::{camera::CameraBuffer, light::LightBuffer};

pub struct RendererBindGroupsLayout {
    global: wgpu::BindGroupLayout,
    water_render: wgpu::BindGroupLayout,
    water_compute: wgpu::BindGroupLayout,
    duck: wgpu::BindGroupLayout,
}

impl RendererBindGroupsLayout {
    pub fn new(device: &wgpu::Device) -> Self {
        RendererBindGroupsLayout {
            global: Self::global_layout(device),
            water_render: Self::water_render_layout(device),
            water_compute: Self::water_compute_layout(device),
            duck: Self::duck_layout(device),
        }
    }

    fn global_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Global Bind Group Layout"),
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
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    fn water_render_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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

    fn water_compute_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Water Compute Layout"),
            entries: &[
                // Current heights
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
                // Previous heights
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Next heights
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Damping
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Normal texture
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        })
    }

    fn duck_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Model Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    pub fn global(&self) -> &wgpu::BindGroupLayout {
        &self.global
    }

    pub fn water(&self) -> &wgpu::BindGroupLayout {
        &self.water_render
    }

    pub fn duck(&self) -> &wgpu::BindGroupLayout {
        &self.duck
    }

    pub fn water_compute(&self) -> &wgpu::BindGroupLayout {
        &self.water_compute
    }
}

pub struct GlobalBindGroup {
    bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    lighting_buffer: wgpu::Buffer,
}

impl GlobalBindGroup {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: size_of::<CameraBuffer>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let lighting_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Lighting Buffer"),
            size: size_of::<LightBuffer>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: lighting_buffer.as_entire_binding(),
                },
            ],
            label: Some("Global Bind Group"),
        });

        GlobalBindGroup {
            bind_group,
            camera_buffer,
            lighting_buffer,
        }
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, buffer: CameraBuffer) {
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[buffer]));
    }

    pub fn update_lighting(&self, queue: &wgpu::Queue, buffer: LightBuffer) {
        queue.write_buffer(&self.lighting_buffer, 0, bytemuck::cast_slice(&[buffer]));
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

pub struct WaterComputeBindGroup {
    bind_group_a: wgpu::BindGroup,
    bind_group_b: wgpu::BindGroup,
    bind_group_c: wgpu::BindGroup,
    buf_a: wgpu::Buffer,
    buf_b: wgpu::Buffer,
    buf_c: wgpu::Buffer,
    _buf_damp: wgpu::Buffer,
    normal_texture: wgpu::Texture,
    frame: usize,
}

impl WaterComputeBindGroup {
    pub const BUFFER_SIZE: usize = 256;

    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let buf_a = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Water buf_a"),
            size: (Self::BUFFER_SIZE * Self::BUFFER_SIZE * size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let buf_b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Water buf_b"),
            size: (Self::BUFFER_SIZE * Self::BUFFER_SIZE * size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let buf_c = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Water buf_c"),
            size: (Self::BUFFER_SIZE * Self::BUFFER_SIZE * size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let damp_data = Self::compute_damping();
        let buf_damp = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Water buf_damp"),
            contents: bytemuck::cast_slice(&damp_data),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let normal_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Water normals"),
            size: wgpu::Extent3d {
                width: Self::BUFFER_SIZE as _,
                height: Self::BUFFER_SIZE as _,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let normal_view = normal_texture.create_view(&Default::default());

        let make_bg = |current: &wgpu::Buffer,
                       previous: &wgpu::Buffer,
                       next: &wgpu::Buffer,
                       label: Option<&'static str>| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: current.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: previous.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: next.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: buf_damp.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&normal_view),
                    },
                ],
                label,
            })
        };

        let bind_group_a = make_bg(&buf_a, &buf_b, &buf_c, Some("Water Compute bind group A"));
        let bind_group_b = make_bg(&buf_c, &buf_a, &buf_b, Some("Water Compute bind group B"));
        let bind_group_c = make_bg(&buf_b, &buf_c, &buf_a, Some("Water Compute bind group C"));

        Self {
            bind_group_a,
            bind_group_b,
            bind_group_c,
            buf_a,
            buf_b,
            buf_c,
            _buf_damp: buf_damp,
            normal_texture,
            frame: 0,
        }
    }

    fn compute_damping() -> Vec<f32> {
        let n = Self::BUFFER_SIZE;
        let h = 2.0 / (n - 1) as f32;
        let mut damping = vec![0.0; n * n];
        for row in 0..n {
            for col in 0..n {
                let x = col as f32 * h - 1.0;
                let y = row as f32 * h - 1.0;
                let l = (1.0 - x.abs()).min(1.0 - y.abs());
                damping[row * n + col] = 0.95 * (l / 0.2).min(1.0);
            }
        }
        damping
    }

    pub fn current_bind_group(&self) -> &wgpu::BindGroup {
        match self.frame % 3 {
            0 => &self.bind_group_a,
            1 => &self.bind_group_b,
            _ => &self.bind_group_c,
        }
    }

    pub fn step(&mut self, encoder: &mut wgpu::CommandEncoder, pipeline: &wgpu::ComputePipeline) {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, self.current_bind_group(), &[]);
        pass.dispatch_workgroups(
            Self::BUFFER_SIZE as u32 / 16,
            Self::BUFFER_SIZE as u32 / 16,
            1,
        );
        drop(pass);
        self.frame += 1;
    }

    pub fn disturb(&self, queue: &wgpu::Queue, col: u32, row: u32, amount: f32) {
        let offset = ((row * Self::BUFFER_SIZE as u32 + col) * size_of::<f32>() as u32) as u64;
        let buf = self.current_buffer();
        queue.write_buffer(buf, offset, bytemuck::cast_slice(&[amount]));
    }

    fn current_buffer(&self) -> &wgpu::Buffer {
        match self.frame % 3 {
            0 => &self.buf_a,
            1 => &self.buf_b,
            _ => &self.buf_c,
        }
    }

    pub fn normal_texture(&self) -> &wgpu::Texture {
        &self.normal_texture
    }
}

pub struct WaterRenderBindGroup {
    bind_group: wgpu::BindGroup,
    cubemap: wgpu::Texture,
}

impl WaterRenderBindGroup {
    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        normal_texture: &wgpu::Texture,
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

        // TODO: this is placeholder
        let cubemap = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Water Cubemap"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
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
            cubemap,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
