use glam::Vec2;

use super::{renderer::GpuState, skin::Skin};

pub struct SVGSkin {
    size: Vec2,
    rotation_center: Vec2,
    rtree: usvg::Tree,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
}

impl SVGSkin {
    pub(super) fn new(gpu_state: &GpuState, svg_data: &[u8], rotation_center: Vec2) -> Self {
        let opt = usvg::Options::default();
        let rtree = usvg::Tree::from_data(svg_data, &opt).unwrap();
        let viewbox_rect = rtree.svg_node().view_box.rect;
        let rotation_center = Vec2::new(
            rotation_center.x - (viewbox_rect.x() as f32),
            rotation_center.y - (viewbox_rect.y() as f32),
        );
        let size = Vec2::new(viewbox_rect.width() as f32, viewbox_rect.height() as f32);

        let mut pixmap = tiny_skia::Pixmap::new(size.x as u32, size.y as u32).unwrap();
        resvg::render(&rtree, usvg::FitTo::Original, pixmap.as_mut()).unwrap();
        pixmap.data();

        let texture_extent = wgpu::Extent3d {
            width: size.x.round() as u32,
            height: size.y.round() as u32,
            depth_or_array_layers: 1,
        };
        let texture = gpu_state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SVGSkin"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        gpu_state.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            pixmap.data(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(size.x.round() as u32 * 4),
                rows_per_image: None,
            },
            texture_extent,
        );

        SVGSkin {
            size,
            rotation_center,
            rtree,
            texture,
            texture_view,
        }
    }
}

impl Skin for SVGSkin {
    fn get_size(&self) -> Vec2 {
        self.size
    }

    fn get_rotation_center(&self) -> Vec2 {
        self.rotation_center
    }

    fn get_texture(&mut self, _scale: f32) -> &wgpu::TextureView {
        &self.texture_view
    }
}
