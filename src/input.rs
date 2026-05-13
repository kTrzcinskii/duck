use glam::Vec2;
use winit::event::{DeviceEvent, ElementState, MouseButton, WindowEvent};

#[derive(Default)]
pub struct InputState {
    pub mouse_delta: Vec2,
    pub lmb_pressed: bool,
    pub rmb_pressed: bool,
}

impl InputState {
    pub fn on_window_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::MouseInput { state, button, .. } = event {
            let pressed = *state == ElementState::Pressed;
            match button {
                MouseButton::Left => self.lmb_pressed = pressed,
                MouseButton::Right => self.rmb_pressed = pressed,
                _ => {}
            }
        }
    }

    pub fn on_device_event(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.mouse_delta += Vec2::new(delta.0 as f32, delta.1 as f32);
        }
    }

    pub fn end_frame(&mut self) {
        self.mouse_delta = Vec2::ZERO;
    }
}
