use std::time::Duration;

use glam::Vec3;

use crate::{
    bind_groups::{RendererBindGroupsLayout, WaterComputeBindGroup},
    light::Light,
    water::{compute::WaterSimulation, render::WaterSurface},
};

pub struct Scene {
    water_simulation: WaterSimulation,
    water_surface: WaterSurface,
    light: Light,
    time_accumulator: f32,
}

impl Scene {
    const WATER_SIMULATION_STEP: f32 = 1.0 / 60.0;

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
        let light = Light::new(Vec3::new(0.0, 3.0, 0.0), Vec3::new(0.5, 0.2, 0.8));
        Scene {
            water_simulation,
            water_surface,
            light,
            time_accumulator: 0.0,
        }
    }

    pub fn update(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        dt: Duration,
    ) {
        self.time_accumulator += dt.as_secs_f32();
        self.update_water(encoder, queue);
    }

    fn update_water(&mut self, encoder: &mut wgpu::CommandEncoder, queue: &wgpu::Queue) {
        const CHANCE_FOR_DROP: f32 = 0.03;
        const DROP_AMOUNT: f32 = 0.25;

        while self.time_accumulator >= Self::WATER_SIMULATION_STEP {
            if rand::random::<f32>() < CHANCE_FOR_DROP {
                let col = rand::random::<u32>() % WaterComputeBindGroup::BUFFER_SIZE as u32;
                let row = rand::random::<u32>() % WaterComputeBindGroup::BUFFER_SIZE as u32;
                self.water_simulation.disturb(queue, col, row, DROP_AMOUNT);
            }
            self.water_simulation.step(encoder);
            self.time_accumulator -= Self::WATER_SIMULATION_STEP;
        }
    }

    pub fn water_surface(&self) -> &WaterSurface {
        &self.water_surface
    }

    pub fn light(&self) -> &Light {
        &self.light
    }
}
