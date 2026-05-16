use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};

pub struct Model {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Model {
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Model {
            translation,
            rotation,
            scale,
        }
    }

    pub fn transform(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    pub fn set_translation(&mut self, translation: Vec3) {
        self.translation = translation;
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ModelBuffer {
    pub model_mtx: [[f32; 4]; 4],
    // 3x3 padded to 4x4 for GPU alignment
    pub normal_mtx: [[f32; 4]; 4],
}

impl From<&Model> for ModelBuffer {
    fn from(value: &Model) -> Self {
        let model_mtx = value.transform();
        let normal_mtx = model_mtx.inverse().transpose();
        ModelBuffer {
            model_mtx: model_mtx.to_cols_array_2d(),
            normal_mtx: normal_mtx.to_cols_array_2d(),
        }
    }
}
