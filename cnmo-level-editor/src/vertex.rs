use eframe::egui_wgpu;
use egui_wgpu::wgpu;

#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn new(pos: (f32, f32, f32), tex_coords: (f32, f32)) -> Self {
        Self { pos: [pos.0, pos.1, pos.2], tex_coords: [tex_coords.0, tex_coords.1] }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2
                },
            ],
        }
    }
}
