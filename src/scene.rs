use std::time::Duration;

use glam::Vec3;

use crate::{
    bind_groups::RendererBindGroupsLayout,
    cubemap::Cubemap,
    light::Light,
    water::{
        compute::{WaterComputeBindGroup, WaterSimulation},
        render::WaterSurface,
    },
};

pub struct Scene {
    water_simulation: WaterSimulation,
    water_surface: WaterSurface,
    cubemap: Cubemap,
    light: Light,
    time_accumulator: f32,
}

impl Scene {
    const WATER_SIMULATION_STEP: f32 = 1.0 / 45.0;

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layouts: &RendererBindGroupsLayout,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let water_simulation = WaterSimulation::new(device, layouts);
        let water_surface = WaterSurface::new(
            device,
            queue,
            layouts,
            water_simulation.normal_texture(),
            surface_format,
        );
        let cubemap = Cubemap::new(device, layouts, surface_format);
        let light = Light::new(Vec3::new(0.0, 3.0, 0.0), Vec3::new(0.5, 0.2, 0.8));
        Scene {
            water_simulation,
            water_surface,
            cubemap,
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
        const DROP_MIN_AMOUNT: f32 = 0.1;

        const DROP_MAX_ADDITION: f32 = 0.3;

        while self.time_accumulator >= Self::WATER_SIMULATION_STEP {
            if rand::random::<f32>() < CHANCE_FOR_DROP {
                let col = rand::random::<u32>() % WaterComputeBindGroup::BUFFER_SIZE as u32;
                let row = rand::random::<u32>() % WaterComputeBindGroup::BUFFER_SIZE as u32;
                let amount = DROP_MIN_AMOUNT + rand::random::<f32>() * DROP_MAX_ADDITION;
                self.water_simulation.disturb(queue, col, row, amount);
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

    pub fn cubemap(&self) -> &Cubemap {
        &self.cubemap
    }
}
