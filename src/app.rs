use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use log::info;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

pub struct AppContext {
    window: Arc<Window>,
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

        match event {
            WindowEvent::CloseRequested => {
                info!("Application stopped");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let dt = ctx.delta_time();
                ctx.window.request_redraw();
            }
            _ => {}
        }
    }
}
