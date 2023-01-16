use eframe::{egui_wgpu, egui, wgpu::util::DeviceExt};
use egui_wgpu::wgpu;
use crate::camera::Camera;
use crate::common_gfx;
use crate::vertex;
use crate::camera;

#[macro_export]
macro_rules! create_instance_resource {
    ($name:ident) => {
        #[derive(Clone, Copy)]
        pub struct $name;
        unsafe impl std::marker::Sync for $name {}
        unsafe impl std::marker::Send for $name {}
    };
}

#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Sprite {
    pub pos: [f32; 3],
    pub size: [f32; 2],
    pub src: [f32; 4],
    pub pure_color: u32,
    pub tint: [f32; 4],
}

impl Sprite {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = 
        wgpu::vertex_attr_array![2 => Float32x3, 3 => Float32x2, 4 => Float32x4, 5 => Uint32, 6 => Float32x4];

    pub fn new(pos: (f32, f32, f32), size: (f32, f32), src: (f32, f32, f32, f32)) -> Self {
        Self { pos: [pos.0, pos.1, pos.2], size: [size.0, size.1], src: [src.0, src.1, src.2, src.3], pure_color: 0, tint: [1.0, 1.0, 1.0, 1.0], }
    }
    
    #[allow(unused)]
    pub fn new_pure_color(pos: (f32, f32, f32), size: (f32, f32), color: (f32, f32, f32, f32)) -> Self {
        Self { pos: [pos.0, pos.1, pos.2], size: [size.0, size.1], src: [0.0, 0.0, 0.0, 0.0], pure_color: 1, tint: [color.0, color.1, color.2, color.3], }
    }

    #[allow(unused)]
    pub fn new_rect(min: (f32, f32), max: (f32, f32), thickness: f32, color: (f32, f32, f32, f32)) -> [Sprite; 4] {
        [
            Self::new_pure_color((min.0, min.1, 0.0), (max.0 - min.0, thickness), color),
            Self::new_pure_color((min.0, max.1 - thickness, 0.0), (max.0 - min.0, thickness), color),
            Self::new_pure_color((min.0, min.1 + thickness, 0.0), (thickness, max.1 - min.1 - thickness * 2.0), color),
            Self::new_pure_color((max.0 - thickness, min.1 + thickness, 0.0), (thickness, max.1 - min.1 - thickness * 2.0), color),
        ]
    }

    #[allow(unused)]
    pub fn create_string(origin_x: f32, origin_y: f32, size: f32, s: &str, sprites: &mut Vec<Sprite>) {
        let mut x = origin_x;
        for char in s.chars() {
            if !char.is_ascii() {
                continue;
            }
            let index = char as u8;
            let src_x = 384 + ((index as u32 % 16) * 8);
            let src_y = 448 + ((index as u32 / 16) * 8);
            sprites.push(Sprite::new(
                (x, origin_y, 0.0),
                (size, size),
                (src_x as f32, src_y as f32, 8.0, 8.0),
            ));
            x += size;
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct InstancedSpritesResources<T: std::marker::Send + std::marker::Sync + Copy + Clone + 'static> {
    pub render_pipeline: wgpu::RenderPipeline,
    pub instance_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub num_instances: usize,
    pub num_verticies: usize,
    pub num_indicies: usize,
    ty: std::marker::PhantomData<T>,
}

impl<T: std::marker::Send + std::marker::Sync + Copy + Clone + 'static> InstancedSpritesResources<T> {
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
            vertex::Vertex::new((-0.0001, -0.0001, 0.0), (0.00002, 0.00002)), // Top-left
            vertex::Vertex::new(( 1.0001, -0.0001, 0.0), (0.99998, 0.00002)), // Top-right
            vertex::Vertex::new((-0.0001,  1.0001, 0.0), (0.00002, 0.99998)), // Bottom-left
            vertex::Vertex::new(( 1.0001,  1.0001, 0.0), (0.99998, 0.99998)), // Bottom-right
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
            Sprite::new((-1.0, 1.0, 0.0), (0.8, -0.8), (64.0, 0.0, 32.0, 32.0)),
            Sprite::new((0.3, 1.0, 0.0), (0.8, -0.8),  (32.0, 0.0, 32.0, 32.0)),
            Sprite::new((-1.0, -0.3, 0.0), (0.8, -0.8),(0.0, 32.0, 32.0, 32.0)),
            Sprite::new((0.3, -0.3, 0.0), (0.8, -0.8), (32.0, 32.0, 32.0, 32.0)),
        ];
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instanced_sprites_instance_buffer"),
            contents: bytemuck::cast_slice(default_sprite_data),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });

        let camera_uniform = camera::CameraUniform::new();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("isntanced_sprites_camera_uniform_buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("instanced_sprites_camera_uniform_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                },
            ],
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("instanced_sprites_camera_uniform_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding()
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("instanced_sprites.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("instanced_sprites_pipeline_layout"),
            bind_group_layouts: &[
                &gfx_common.texture_bind_group_layout,
                &camera_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("instanced_sprites_render_pipeline"),
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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        paint_callback_resources.insert(InstancedSpritesResources::<T> {
            render_pipeline,
            instance_buffer,
            vertex_buffer,
            index_buffer,
            camera_buffer,
            camera_bind_group,
            num_indicies: sprite_indicies.len(),
            num_verticies: sprite_verticies.len(),
            num_instances: default_sprite_data.len(),
            ty: std::marker::PhantomData,
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
        self.num_instances = sprites.len();
    }

    #[allow(unused)]
    pub fn update_sprite(&self, queue: &wgpu::Queue, idx: usize, sprite: Sprite) {
        queue.write_buffer(&self.instance_buffer, idx as _, bytemuck::cast_slice(&[sprite]));
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &camera::Camera) {
        let camera_uniform = camera::CameraUniform::from_view_proj(camera);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
    }

    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>, texture: &'rp wgpu::BindGroup) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, texture, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indicies as _, 0, 0..self.num_instances as _);
    }
}

pub struct InstancedSprites {
    sprites: Option<Vec<Sprite>>,
    camera: Option<Camera>,
}

impl InstancedSprites {
    pub fn new() -> Self {
        Self {
            sprites: None,
            camera: None,
        }
    }

    pub fn with_sprites(self, sprites: Vec<Sprite>) -> Self {
        Self {
            sprites: Some(sprites),
            camera: self.camera,
        }
    }

    pub fn with_camera(self, camera: Camera) -> Self {
        Self {
            sprites: self.sprites,
            camera: Some(camera),
        }
    }

    pub fn paint<R: std::marker::Send + std::marker::Sync + Copy + Clone + 'static>(self, ui: &mut egui::Ui, rect: egui::Rect) {
        let cb = egui_wgpu::CallbackFn::new()
            .prepare(move |device, queue, _encoder, paint_callback_resources| {
                let resources: &mut InstancedSpritesResources::<R> = match paint_callback_resources.get_mut() {
                    Some(resources) => resources,
                    None => {
                        log::warn!("Can't find InstancedSpritesResources!");
                        return Vec::new();
                    },
                };
                if let Some(sprites) = &self.sprites {
                    resources.buffer_sprites(device, queue, &sprites);
                }
                if let Some(camera) = &self.camera {
                    resources.update_camera(queue, camera);
                } else {
                    resources.update_camera(queue, &camera::Camera::new());
                }
                Vec::new()
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let resources: &InstancedSpritesResources::<R> = match paint_callback_resources.get() {
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
