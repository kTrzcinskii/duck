use wgpu::util::DeviceExt;

use crate::bind_groups::RendererBindGroupsLayout;

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

    pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
        queue.write_buffer(
            self.current_buffer(),
            offset,
            bytemuck::cast_slice(&[amount]),
        );
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
