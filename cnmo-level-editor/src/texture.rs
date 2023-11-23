use std::io::Cursor;

use eframe::egui_wgpu::wgpu;

pub struct Texture {
    pub sampler: wgpu::Sampler,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub palette: Vec<[u8; 3]>,
    pub dimensions: (u32, u32),
    pub opaques: Vec<Vec<bool>>,
}

struct ImageData {
    pub palette: Vec<[u8; 3]>,
    pub image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    pub size: (u32, u32),
    pub opaques: Vec<Vec<bool>>,
}

static EDITOR_BASE: &'static [u8] = include_bytes!("editorbase.bmp");

impl ImageData {
    fn from_file<P: AsRef<std::path::Path>>(path: P) -> Option<Self> {
        Self::get_image(&Self::get_buffer_from_file(path))
    }

    fn get_buffer_from_file<P: AsRef<std::path::Path>>(path: P) -> Vec<u8> {
        match std::fs::read(path.as_ref()) {
            Ok(ref buffer) => buffer.to_vec(),
            Err(_) => {
                log::error!("Can't open image file: {:?}.", path.as_ref());
                log::error!("Loading backup image file instead.");
                include_bytes!("gfx_backup.bmp").to_vec()
            },
        }
    }

    fn compute_opaques(rgba: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) -> Vec<Vec<bool>> {
        let (width, height) = rgba.dimensions();
        let mut opaques = (0..width).map(|_| { (0..height).map(|_| { true }).collect::<Vec<bool>>() }).collect::<Vec<Vec<bool>>>();

        for y in 0..height {
            for x in 0..width {
                let rgba = rgba.get_pixel(x, y).0;
                opaques[x as usize][y as usize] = rgba[0] == 0 && rgba[1] == 255 && rgba[2] == 255 && rgba[3] == 255;
            }
        }

        return opaques;
    }

    fn get_image(buffer: &[u8]) -> Option<Self> {
        let decoder = match image::codecs::bmp::BmpDecoder::new(Cursor::new(buffer)) {
            Ok(decoder) => decoder,
            Err(_) => {
                log::warn!("not a bmp file!");
                return None;
            },
        };
        let image = image::load_from_memory(buffer).unwrap();
        let rgba = image.to_rgba8();
        Some(Self {
            palette: match decoder.get_palette() {
                Some(slice) => slice.to_owned(),
                None => Vec::new(),
            },
            opaques: Self::compute_opaques(&rgba),
            image: rgba,
            size: <image::DynamicImage as image::GenericImageView>::dimensions(&image),
        })
    }

    pub fn construct_editorimage(self) -> (Self, u32) {
        let editorbase = Self::get_image(EDITOR_BASE).unwrap();
        let mut newimage = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::new(self.size.0, self.size.1 + editorbase.size.1);
        
        // Put the base image here
        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                newimage.put_pixel(x, y, *self.image.get_pixel(x, y));
            }
        }

        // Add on the editorbase thing
        for y in 0..editorbase.size.1 {
            for x in 0..editorbase.size.0 {
                newimage.put_pixel(x, y + self.size.1, *editorbase.image.get_pixel(x, y));
            }
        }

        (
            Self {
                palette: self.palette,
                opaques: Self::compute_opaques(&newimage),
                image: newimage,
                size: self.size,
            },
            self.size.1 + editorbase.size.1
        )
    }
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
            opaques: vec![],
            palette: Vec::new(),
            dimensions: (size.clone().into().0, size.clone().into().1),
        }
    }

    pub fn from_file<P: AsRef<std::path::Path>>(device: &wgpu::Device, queue: &wgpu::Queue, path: P) -> Self {
        match ImageData::from_file(path) {
            Some(image) => Self::from_memory(device, queue, image.construct_editorimage()),
            None => Self::new(device, None, (1, 1)),
        }
    }

    fn from_memory(device: &wgpu::Device, queue: &wgpu::Queue, image: (ImageData, u32)) -> Self {
        let size = (image.0.size.0, image.1);
        let mut texture = Self::new(device, Some("texture"), size);
        texture.palette = image.0.palette;
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image.0.image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * size.0),
                rows_per_image: std::num::NonZeroU32::new(size.1),
            },
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );
        texture.dimensions = image.0.size;
        texture.opaques = image.0.opaques;
        texture
    }
}
