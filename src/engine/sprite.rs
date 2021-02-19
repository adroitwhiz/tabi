use std::cell::RefCell;

use crate::engine::target::Target;
use crate::engine::thread::Thread;

pub struct Sprite<'a> {
    pub x: f64,
    pub y: f64,
    pub direction: f64,
    pub size: f64,
    pub visible: bool,
    pub threads: Vec<RefCell<Thread<'a>>>,
    pub target: &'a Target,
}

impl<'a> Sprite<'a> {
    pub fn new(target: &'a Target) -> Self {
        // Create a new thread for every script
        let threads: Vec<RefCell<Thread>> = target
            .scripts
            .iter()
            .map(|s| { RefCell::new(Thread::new(s)) })
            .collect();

        Sprite {
            x: 0.0,
            y: 0.0,
            direction: 0.0,
            size: 100.0,
            visible: true,
            threads,
            target,
        }
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }
}
