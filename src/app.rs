use std::{
    f32::consts::{self},
    sync::Arc,
    time::{Duration, Instant},
};

use log::{error, info};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::{
    camera::{CameraBuffer, OrbitCamera, Projection},
    input::InputState,
    renderer::Renderer,
    scene::Scene,
};

pub struct AppContext {
    window: Arc<Window>,
    input: InputState,
    last_render_time: Instant,
    renderer: Renderer,
    scene: Scene,
    camera: OrbitCamera,
    projection: Projection,
}

impl AppContext {
    fn delta_time(&mut self) -> Duration {
        let now = Instant::now();
        let dt = now - self.last_render_time;
        self.last_render_time = now;
        dt
    }

    fn on_input_update(&mut self) {
        self.camera.update(&self.input);
    }

    fn render(&mut self, dt: Duration) {
        let camera_buffer = CameraBuffer::new(&self.camera, &self.projection);
        if let Err(e) = self.renderer.render(camera_buffer, &mut self.scene, dt) {
            error!("Failed to render a frame: {e}");
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
        self.projection.resize(width, height);
    }
}

#[derive(Default)]
pub struct App {
    ctx: Option<AppContext>,
}

const CAMERA_RADIUS: f32 = 5.0;

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.ctx.is_some() {
            return;
        }

        let window_attrs = WindowAttributes::default()
            .with_title("Duck")
            .with_maximized(true);
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        let renderer = match pollster::block_on(Renderer::new(window.clone())) {
            Ok(r) => r,
            Err(err) => {
                error!("Failed to construct Renderer: {err}");
                return;
            }
        };

        let surface_config = renderer.surface_config();
        let scene = Scene::new(renderer.device(), renderer.layouts(), surface_config.format);

        let camera = OrbitCamera::new(CAMERA_RADIUS);
        let projection = Projection::new(
            surface_config.width,
            surface_config.height,
            consts::FRAC_PI_4,
            0.1,
            100.0,
        );

        let ctx = AppContext {
            window,
            input: InputState::default(),
            last_render_time: Instant::now(),
            renderer,
            scene,
            camera,
            projection,
        };

        self.ctx = Some(ctx);

        info!("Application started");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let ctx = match &mut self.ctx {
            Some(ctx) => ctx,
            None => return,
        };

        ctx.input.on_window_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                info!("Application stopped");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let dt = ctx.delta_time();
                ctx.on_input_update();
                ctx.render(dt);
                ctx.input.end_frame();
                ctx.window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                log::info!("Resize: {}x{}", size.width, size.height);
                ctx.resize(size.width, size.height);
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(ctx) = &mut self.ctx {
            ctx.input.on_device_event(&event);
        }
    }
}
