use std::io::Cursor;

use eframe::egui_wgpu::wgpu;

pub struct Texture {
    pub sampler: wgpu::Sampler,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub palette: Vec<[u8; 3]>,
    pub dimensions: (u32, u32),
}

impl Texture {
    pub fn new<S: Into<(u32, u32)> + Clone>(device: &wgpu::Device, label: Option<&str>, size: S) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: size.clone().into().0,
                height: size.clone().into().1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            palette: Vec::new(),
            dimensions: (size.clone().into().0, size.clone().into().1),
        }
    }

    pub fn from_file<P: AsRef<std::path::Path>>(device: &wgpu::Device, queue: &wgpu::Queue, path: P) -> Self {
        Self::from_memory(device, queue, match std::fs::read(path.as_ref()) {
            Ok(ref buffer) => buffer.as_slice(),
            Err(_) => {
                log::error!("Can't open image file: {:?}.", path.as_ref());
                log::error!("Loading backup image file instead.");
                include_bytes!("gfx_backup.bmp")
            },
        })
    }

    pub fn from_memory(device: &wgpu::Device, queue: &wgpu::Queue, buffer: &[u8]) -> Self {
        let decoder = match image::codecs::bmp::BmpDecoder::new(Cursor::new(buffer)) {
            Ok(decoder) => decoder,
            Err(_) => {
                log::warn!("not a bmp file!");
                return Self::new(device, None, (1, 1));
            },
        };
        let image = image::load_from_memory(buffer).unwrap();
        let rgba = image.to_rgba8();
        let dimensions = <image::DynamicImage as image::GenericImageView>::dimensions(&image);
        let mut texture = Self::new(device, Some("texture"), dimensions);
        texture.palette = match decoder.get_palette() {
            Some(slice) => slice.to_owned(),
            None => Vec::new(),
        };
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            },
        );
        texture
    }
}
