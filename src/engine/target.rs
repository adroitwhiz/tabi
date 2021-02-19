use crate::engine::instruction::Script;

/// The "prototype" of a sprite. Each Sprite object is an instance that refers back to one of these.
#[derive(Debug)]
pub struct Target {
    pub scripts: Vec<Script>,
    pub is_stage: bool,
    pub name: String,
    pub layer_order: u32,
}
