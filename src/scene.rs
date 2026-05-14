use crate::{
    bind_groups::{RendererBindGroupsLayout, WaterComputeBindGroup},
    water::{compute::WaterSimulation, render::WaterSurface},
};

pub struct Scene {
    water_simulation: WaterSimulation,
    water_surface: WaterSurface,
}

impl Scene {
    pub fn new(
        device: &wgpu::Device,
        layouts: &RendererBindGroupsLayout,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let water_simulation = WaterSimulation::new(device, layouts);
        let water_surface = WaterSurface::new(
            device,
            layouts,
            water_simulation.normal_texture(),
            surface_format,
        );
        Scene {
            water_simulation,
            water_surface,
        }
    }

    pub fn update(&mut self, encoder: &mut wgpu::CommandEncoder, queue: &wgpu::Queue) {
        const CHANCE_FOR_DROP: f32 = 0.03;
        const DROP_AMOUNT: f32 = 0.25;
        if rand::random::<f32>() < CHANCE_FOR_DROP {
            let col = rand::random::<u32>() % WaterComputeBindGroup::BUFFER_SIZE as u32;
            let row = rand::random::<u32>() % WaterComputeBindGroup::BUFFER_SIZE as u32;
            self.water_simulation.disturb(queue, col, row, DROP_AMOUNT);
        }
        self.water_simulation.step(encoder);
    }

    pub fn water_surface(&self) -> &WaterSurface {
        &self.water_surface
    }
}
