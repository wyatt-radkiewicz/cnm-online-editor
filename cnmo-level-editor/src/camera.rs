use eframe::egui_wgpu;
use egui_wgpu::wgpu;
use cgmath::{SquareMatrix, Transform};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    pub pos: cgmath::Vector2<f32>,
    pub zoom: f32,
    pub ang: f32,
    pub proj: cgmath::Matrix4<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            pos: cgmath::vec2(0.0, 0.0),
            zoom: 1.0,
            ang: 0.0,
            proj: cgmath::Matrix4::identity(),
        }
    }

    pub fn set_projection(&mut self, mut width: f32, mut height: f32, fixed_height: Option<f32>, origin_topleft: bool) {
        if let Some(fixed_height) = fixed_height {
            let aspect_ratio = width / height;
            height = fixed_height;
            width = fixed_height * aspect_ratio;
        }

        let left = if origin_topleft { 0.0 } else { -width / 2.0 };
        let top = if origin_topleft { 0.0 } else { height / 2.0 };
        self.proj = cgmath::ortho(left, left+width, top+height, top, 0.0, 100.0);
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::from_angle_z(cgmath::Rad(-self.ang)) *
            cgmath::Matrix4::from_scale(self.zoom) *
            cgmath::Matrix4::from_translation((-self.pos.x, -self.pos.y, 0.0).into());
        OPENGL_TO_WGPU_MATRIX * self.proj * view
    }
}

#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct CameraUniform {
    pub matrix: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            matrix: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.matrix = camera.build_view_projection_matrix().into();
    }
}
