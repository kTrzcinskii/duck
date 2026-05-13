use bytemuck::{Pod, Zeroable};
use glam::Vec3;

pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
}

impl Light {
    pub fn new(position: Vec3, color: Vec3) -> Self {
        Self { position, color }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct LightBuffer {
    pub position: [f32; 4],
    pub color: [f32; 4],
}

impl From<&Light> for LightBuffer {
    fn from(light: &Light) -> Self {
        Self {
            position: [light.position.x, light.position.y, light.position.z, 1.0],
            color: [light.color.x, light.color.y, light.color.z, 1.0],
        }
    }
}
