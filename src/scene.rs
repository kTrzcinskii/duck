use std::time::Duration;

use glam::Vec3;

use crate::{
    bind_groups::RendererBindGroupsLayout,
    cubemap::Cubemap,
    duck::{controller::DuckController, render::Duck},
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
    duck: Duck,
    duck_controller: DuckController,
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
        let duck = Duck::new(device, layouts, surface_format, queue);
        let light = Light::new(Vec3::new(0.0, 1.5, 0.0), Vec3::new(1.0, 1.0, 1.0));
        Scene {
            water_simulation,
            water_surface,
            cubemap,
            duck,
            duck_controller: DuckController::default(),
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
        self.update_duck(queue, dt);
        self.update_water(encoder, queue);
    }

    fn update_duck(&mut self, queue: &wgpu::Queue, dt: Duration) {
        const DUCK_DISTURB_AMOUNT: f32 = 0.15;
        let (position, rotation) = self.duck_controller.update(dt.as_secs_f32());
        self.duck.update_model(queue, |model| {
            model.set_translation(position);
            model.set_rotation(rotation);
        });
        let (col, row) = self.duck_controller.grid_position();
        self.water_simulation
            .disturb(queue, col, row, DUCK_DISTURB_AMOUNT);
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

    pub fn duck(&self) -> &Duck {
        &self.duck
    }
}
