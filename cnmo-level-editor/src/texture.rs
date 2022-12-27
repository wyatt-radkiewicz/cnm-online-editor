use eframe::egui_wgpu::wgpu;

pub struct Texture {
    pub sampler: wgpu::Sampler,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
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
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
        }
    }

    pub fn from_file<P: AsRef<std::path::Path>>(device: &wgpu::Device, queue: &wgpu::Queue, path: P) -> Self {
        let image = match std::fs::read(path.as_ref()) {
            Ok(contents) => image::load_from_memory(&contents),
            Err(err) => Err(image::ImageError::IoError(err)),
        };

        if image.is_err() {
            log::warn!("Can't load the texture from the path: {}", path.as_ref().as_os_str().to_string_lossy());
            return Self::new(device, Some("texture"), (1, 1));
        }

        let image = image.unwrap();
        let rgba = image.to_rgba8();
        let dimensions = <image::DynamicImage as image::GenericImageView>::dimensions(&image);
        let texture = Self::new(device, Some("texture"), dimensions);
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
