use crate::engine::target::Target;

pub struct Sprite<'a> {
    pub x: f64,
    pub y: f64,
    pub direction: f64,
    pub size: f64,
    pub visible: bool,
    pub thread_indices: Box<[usize]>,
    pub target: &'a Target,
}

impl<'a> Sprite<'a> {
    pub fn new(target: &'a Target, thread_indices: Box<[usize]>) -> Self {
        Sprite {
            x: 0.0,
            y: 0.0,
            direction: 0.0,
            size: 100.0,
            visible: true,
            thread_indices,
            target,
        }
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }
}
