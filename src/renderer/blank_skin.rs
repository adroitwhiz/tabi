use glam::Vec2;
use wgpu::TextureView;

use super::skin::Skin;

pub struct BlankSkin {
    size: Vec2,
    rotation_center: Vec2,
}

impl BlankSkin {
    pub fn new(size: Vec2, rotation_center: Vec2) -> Self {
        BlankSkin { size, rotation_center }
    }
}

impl Skin for BlankSkin {
    fn get_size(&self) -> Vec2 {
        self.size
    }

    fn get_rotation_center(&self) -> Vec2 {
        self.rotation_center
    }

    fn get_texture(&mut self, _scale: f32) -> &TextureView {
        unimplemented!()
    }
}
