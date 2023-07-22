use std::{num::NonZeroU32, sync::Mutex};

use once_cell::sync::Lazy;

use super::resource::LoadedResource;

pub type TextureId = u8;

static TEXTURE_SET: Lazy<Mutex<Vec<image::DynamicImage>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn add_texture(texture_resource: LoadedResource) -> Result<TextureId, &'static str> {
    let LoadedResource::TEXTURE(image) = texture_resource else { return Err("A texture resource wasn't supplied to add_texture!") };
    let mut texture_set = TEXTURE_SET.lock().unwrap();
    texture_set.push(image.flipv());
    Ok((texture_set.len() - 1) as TextureId)
}

pub fn create_texture_array(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_width: u32,
    texture_height: u32,
) -> wgpu::Texture {
    let texture_set = TEXTURE_SET.lock().unwrap();
    let texture_count = texture_set.len() as u32;
    let texture_extent = wgpu::Extent3d {
        width: texture_width,
        height: texture_height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: texture_width,
            height: texture_height,
            depth_or_array_layers: texture_count,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for i in 0..texture_count {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: i },
                aspect: wgpu::TextureAspect::All,
            },
            &unsafe { texture_set.get_unchecked(i as usize).to_rgba8() },
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * texture_width),
                rows_per_image: NonZeroU32::new(texture_height),
            },
            texture_extent,
        );
    }
    texture
}
