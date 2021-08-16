use core::fmt;

use glam::Vec2;
use wgpu::TextureView;

pub trait Skin {
    fn get_texture(&mut self, scale: f32) -> &TextureView;
    fn get_size(&self) -> Vec2;
    fn get_rotation_center(&self) -> Vec2;
}

impl fmt::Debug for dyn Skin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Skin")
    }
}
