use cgmath::SquareMatrix;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(Clone)]
pub struct Camera {
    pub pos: cgmath::Vector2<f32>,
    pub zoom: f32,
    pub ang: f32,
    pub proj: cgmath::Matrix4<f32>,
    is_gui_proj: bool,
    proj_size: cgmath::Vector2<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            pos: cgmath::vec2(0.0, 0.0),
            zoom: 1.0,
            ang: 0.0,
            proj: cgmath::Matrix4::identity(),
            is_gui_proj: false,
            proj_size: cgmath::vec2(2.0, 2.0)
        }
    }

    pub fn with_projection(mut self, width: f32, height: f32, fixed_height: Option<f32>, origin_topleft: bool) -> Self {
        self.set_projection(width, height, fixed_height, origin_topleft);
        self
    }

    pub fn set_projection(&mut self, mut width: f32, mut height: f32, fixed_height: Option<f32>, origin_topleft: bool) {
        self.is_gui_proj = origin_topleft;
        if let Some(fixed_height) = fixed_height {
            let aspect_ratio = width / height;
            height = fixed_height;
            width = fixed_height * aspect_ratio;
        }

        self.proj_size = cgmath::vec2(width, height);
        let left = if origin_topleft { 0.0 } else { -width / 2.0 };
        let bottom = if origin_topleft { height } else { height / 2.0 };
        let top = if origin_topleft { 0.0 } else { -height / 2.0 };
        self.proj = cgmath::ortho(left, left+width, bottom, top, 1000.0, 0.0);
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::from_angle_z(cgmath::Rad(-self.ang)) *
            cgmath::Matrix4::from_scale(self.zoom) *
            cgmath::Matrix4::from_translation((-self.pos.x, -self.pos.y, 0.0).into());
        OPENGL_TO_WGPU_MATRIX * self.proj * view
    }

    pub fn get_top_left_world_space(&self) -> cgmath::Vector2<f32> {
        if self.is_gui_proj {
            self.pos
        } else {
            self.pos - self.proj_size / 2.0 / self.zoom
        }
    }

    pub fn get_bottom_right_world_space(&self) -> cgmath::Vector2<f32> {
        if self.is_gui_proj {
            self.pos + self.proj_size / self.zoom
        } else {
            self.pos + self.proj_size / 2.0 / self.zoom
        }
    }

    pub fn get_proj_size_world_space(&self) -> cgmath::Vector2<f32> {
        self.proj_size / self.zoom
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

    pub fn from_view_proj(camera: &Camera) -> Self {
        Self {
            matrix: camera.build_view_projection_matrix().into(),
        }
    }
}
