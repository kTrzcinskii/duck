use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use log::info;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::input::InputState;

pub struct AppContext {
    window: Arc<Window>,
    input: InputState,
    last_render_time: Instant,
}

impl AppContext {
    fn delta_time(&mut self) -> Duration {
        let now = Instant::now();
        let dt = now - self.last_render_time;
        self.last_render_time = now;
        dt
    }
}

#[derive(Default)]
pub struct App {
    ctx: Option<AppContext>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.ctx.is_some() {
            return;
        }

        let window_attrs = WindowAttributes::default()
            .with_title("Duck")
            .with_maximized(true);
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        let ctx = AppContext {
            window,
            input: InputState::default(),
            last_render_time: Instant::now(),
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
                ctx.input.end_frame();
                ctx.window.request_redraw();
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
