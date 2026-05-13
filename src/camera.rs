use std::f32::consts;

use glam::{Mat4, Vec3};

use crate::input::InputState;

pub struct OrbitCamera {
    target: Vec3,
    // Horizontal
    theta: f32,
    // Vertical
    phi: f32,
    radius: f32,
    sensitivity: f32,
    scroll_speed: f32,
}

impl OrbitCamera {
    const TARGET: Vec3 = Vec3::ZERO;
    const SENSITIVITY: f32 = 0.005;
    const ZOOM_SPEED: f32 = 0.01;
    const MIN_RADIUS: f32 = 0.1;
    const MIN_PHI: f32 = 0.01;
    const MAX_PHI: f32 = consts::PI - 0.01;

    pub fn new(radius: f32) -> Self {
        Self {
            target: Self::TARGET,
            theta: 0.0,
            phi: consts::FRAC_PI_4,
            radius: radius.max(Self::MIN_RADIUS),
            sensitivity: Self::SENSITIVITY,
            scroll_speed: Self::ZOOM_SPEED,
        }
    }

    pub fn update(&mut self, input: &InputState) {
        if input.lmb_pressed {
            self.theta -= input.mouse_delta.x * self.sensitivity;
            self.phi = (self.phi - input.mouse_delta.y * self.sensitivity)
                .clamp(Self::MIN_PHI, Self::MAX_PHI);
        }
        if input.rmb_pressed {
            self.radius =
                (self.radius + input.mouse_delta.y * self.scroll_speed).max(Self::MIN_RADIUS);
        }
    }

    pub fn position(&self) -> Vec3 {
        let x = self.radius * self.phi.sin() * self.theta.cos();
        let y = self.radius * self.phi.cos();
        let z = self.radius * self.phi.sin() * self.theta.sin();
        self.target + Vec3::new(x, y, z)
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position(), self.target, Vec3::Y)
    }
}

pub struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraBuffer {
    view_projection_matrix: [[f32; 4]; 4],
    inverse_projection_matrix: [[f32; 4]; 4],
    inverse_view_matrix: [[f32; 4]; 4],
}

impl CameraBuffer {
    pub fn new(camera: &OrbitCamera, projection: &Projection) -> Self {
        let view = camera.view_matrix();
        let proj = projection.matrix();
        CameraBuffer {
            view_projection_matrix: (proj * view).to_cols_array_2d(),
            inverse_projection_matrix: proj.inverse().to_cols_array_2d(),
            inverse_view_matrix: view.inverse().to_cols_array_2d(),
        }
    }
}
