use std::rc::Rc;

use crate::engine::target::Target;
use crate::renderer::renderer::{Renderer, DrawableID};

pub struct Sprite<'t> {
    pub x: f64,
    pub y: f64,
    pub direction: f64,
    pub size: f64,
    pub visible: bool,
    pub target: &'t Target,
    pub layer_order: u32,
    pub drawable: DrawableID,
}

impl<'t> Sprite<'t> {
    pub fn new(target: &'t Target, renderer: &mut Renderer) -> Self {
        Sprite {
            x: 0.0,
            y: 0.0,
            direction: 0.0,
            size: 100.0,
            visible: true,
            target,
            layer_order: target.layer_order,
            drawable: renderer.create_drawable(Rc::clone(&target.costumes[0].skin))
        }
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }
}
