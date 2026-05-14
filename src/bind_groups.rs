use crate::{
    camera::CameraBuffer,
    light::LightBuffer,
    water::{compute::WaterComputeBindGroup, render::WaterRenderBindGroup},
};

pub struct RendererBindGroupsLayout {
    global: wgpu::BindGroupLayout,
    water_render: wgpu::BindGroupLayout,
    water_compute: wgpu::BindGroupLayout,
}

impl RendererBindGroupsLayout {
    pub fn new(device: &wgpu::Device) -> Self {
        RendererBindGroupsLayout {
            global: GlobalBindGroup::layout(device),
            water_render: WaterRenderBindGroup::layout(device),
            water_compute: WaterComputeBindGroup::layout(device),
        }
    }

    pub fn global(&self) -> &wgpu::BindGroupLayout {
        &self.global
    }

    pub fn water(&self) -> &wgpu::BindGroupLayout {
        &self.water_render
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
    pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
