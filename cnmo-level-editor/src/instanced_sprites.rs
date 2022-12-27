use eframe::{egui_wgpu, egui, wgpu::util::DeviceExt};
use egui_wgpu::wgpu;
use crate::common_gfx;
use crate::vertex;

#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Sprite {
    pub pos: [f32; 3],
    pub size: [f32; 2],
    pub src: [f32; 4],
}

impl Sprite {
    pub fn new(pos: (f32, f32, f32), size: (f32, f32), src: (f32, f32, f32, f32)) -> Self {
        Self { pos: [pos.0, pos.1, pos.2], size: [size.0, size.1], src: [src.0, src.1, src.2, src.3] }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4
                }
            ],
        }
    }
}

pub struct InstancedSpritesResources {
    pub render_pipeline: wgpu::RenderPipeline,
    pub instance_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_instances: usize,
    pub num_verticies: usize,
    pub num_indicies: usize,
}

impl InstancedSpritesResources {
    pub fn insert_resource(cc: &eframe::CreationContext) {
        let wgpu_render_state = cc.wgpu_render_state.as_ref().expect("Need a WGPU render state for app to function!");
        let device = &wgpu_render_state.device;
        let paint_callback_resources = &mut wgpu_render_state.renderer.write().paint_callback_resources;

        let gfx_common = match paint_callback_resources.get::<common_gfx::GfxCommonResources>() {
            Some(gfx_common) => gfx_common,
            None => {
                log::warn!("Can't find GfxCommonResources");
                return;
            },
        };

        let sprite_verticies = &[
            vertex::Vertex::new((0.0, 0.0, 0.0), (0.0, 0.0)), // Top-left
            vertex::Vertex::new((1.0, 0.0, 0.0), (1.0, 0.0)), // Top-right
            vertex::Vertex::new((0.0, -1.0, 0.0), (0.0, 1.0)), // Bottom-left
            vertex::Vertex::new((1.0, -1.0, 0.0), (1.0, 1.0)), // Bottom-right
        ];
        let sprite_indicies: &[u16] = &[
            0, 2, 1,
            1, 2, 3,
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instanced_sprites_vertex_buffer"),
            contents: bytemuck::cast_slice(sprite_verticies),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instanced_sprites_index_buffer"),
            contents: bytemuck::cast_slice(sprite_indicies),
            usage: wgpu::BufferUsages::INDEX,
        });

        let default_sprite_data = &[
            Sprite::new((-1.0, 1.0, 0.0), (0.5, 0.5), (0.1, 0.1, 0.1, 0.1)),
            Sprite::new((0.5, 1.0, 0.0), (0.5, 0.5), (0.0, 0.0, 0.1, 0.1)),
            Sprite::new((-1.0, -0.5, 0.0), (0.5, 0.5), (0.1, 0.4, 0.1, 0.1)),
            Sprite::new((0.5, -0.5, 0.0), (0.5, 0.5), (0.1, 0.10, 0.03, 0.03)),
        ];
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instanced_sprites_instance_buffer"),
            contents: bytemuck::cast_slice(default_sprite_data),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("instanced_sprites.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("test_triangle_pipeline_layout"),
            bind_group_layouts: &[
                &gfx_common.texture_bind_group_layout
            ],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("test_triangle_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex::Vertex::desc(), Sprite::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu_render_state.target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,//Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        paint_callback_resources.insert(Self {
            render_pipeline,
            instance_buffer,
            vertex_buffer,
            index_buffer,
            num_indicies: sprite_indicies.len(),
            num_verticies: sprite_verticies.len(),
            num_instances: default_sprite_data.len(),
        });
    }

    pub fn buffer_sprites(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, sprites: &[Sprite]) {
        if self.num_instances == sprites.len() {
            queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(sprites));
        } else {
            self.instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instanced_sprites_instance_buffer"),
                contents: bytemuck::cast_slice(sprites),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            });
        }
    }

    pub fn update_sprite(&self, queue: &wgpu::Queue, idx: usize, sprite: Sprite) {
        queue.write_buffer(&self.instance_buffer, idx as _, bytemuck::cast_slice(&[sprite]));
    }

    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>, texture: &'rp wgpu::BindGroup) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, texture, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indicies as _, 0, 0..self.num_instances as _);
    }
}

pub struct InstancedSprites {
    sprites: Option<Vec<Sprite>>,
}

impl InstancedSprites {
    pub fn new() -> Self {
        Self {
            sprites: None,
        }
    }

    pub fn with_sprites(self, sprites: Vec<Sprite>) -> Self {
        Self {
            sprites: Some(sprites),
        }
    }

    pub fn paint(self, ui: &mut egui::Ui, rect: egui::Rect) {
        let cb = egui_wgpu::CallbackFn::new()
            .prepare(move |device, queue, _encoder, paint_callback_resources| {
                if let Some(sprites) = &self.sprites {
                    let resources: &mut InstancedSpritesResources = match paint_callback_resources.get_mut() {
                        Some(resources) => resources,
                        None => {
                            log::warn!("Can't find InstancedSpritesResources!");
                            return Vec::new();
                        },
                    };
                    resources.buffer_sprites(device, queue, &sprites);
                }
                Vec::new()
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let resources: &InstancedSpritesResources = match paint_callback_resources.get() {
                    Some(resources) => resources,
                    None => {
                        log::warn!("Can't find InstancedSpritesResources!");
                        return;
                    },
                };
                let gfx: &common_gfx::GfxCommonResources = match paint_callback_resources.get() {
                    Some(gfx) => gfx,
                    None => {
                        log::warn!("Can't find GfxCommonResources!");
                        return;
                    },
                };
                resources.paint(render_pass, &gfx.texture_bind_group);
            });
        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(cb),
        };
        ui.painter().add(callback);
    }
}
