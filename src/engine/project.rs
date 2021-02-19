use crate::engine::target::Target;

/// The "static" part of a Scratch project. E.g. contains targets (sprites' prototypes), blocks, etc
#[derive(Debug)]
pub struct Project {
    pub targets: Vec<Target>,
}
