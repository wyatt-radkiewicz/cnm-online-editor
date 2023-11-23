use eframe::egui_wgpu;
use egui_wgpu::wgpu;
use crate::texture;

pub struct GfxCommonResources {
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_bind_group: wgpu::BindGroup,
    pub texture: texture::Texture,
}

impl GfxCommonResources {
    pub fn insert_resource<P: AsRef<std::path::Path>>(wgpu_render_state: &egui_wgpu::RenderState, gfx_path: P) -> (Vec<[u8; 3]>, (u32, u32), Vec<Vec<bool>>) {
        let device = &wgpu_render_state.device;
        let queue = &wgpu_render_state.queue;
        let paint_callback_resources = &mut wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources;

        let texture = texture::Texture::from_file(device, queue, gfx_path);
        let texture_bind_group_layout = if paint_callback_resources.contains::<GfxCommonResources>() {
            None
        } else {
            Some(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        count: None,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                    },
                    wgpu::BindGroupLayoutEntry {
                        count: None,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                    },
                ],
            }))
        };
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout: if let Some(ref texture_bind_group_layout) = texture_bind_group_layout {
                texture_bind_group_layout
            } else {
                &paint_callback_resources.get::<GfxCommonResources>().unwrap().texture_bind_group_layout
            },
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture.view)
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler)
            }]
        });

        let palette = texture.palette.clone();
        let dimensions = texture.dimensions.clone();
        let opaques = texture.opaques.clone();

        if paint_callback_resources.contains::<GfxCommonResources>() {
            let resource = paint_callback_resources.get_mut::<GfxCommonResources>().unwrap();
            resource.texture_bind_group = texture_bind_group;
            resource.texture = texture;
        } else {
            paint_callback_resources.insert(GfxCommonResources {
                texture_bind_group_layout: texture_bind_group_layout.unwrap(),
                texture_bind_group,
                texture,
            });
        }

        (palette, dimensions, opaques)
    }
}
