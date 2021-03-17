use crate::{data::asset::Costume, engine::instruction::Script};

/// The "prototype" of a sprite. Each Sprite object is an instance that refers back to one of these.
#[derive(Debug)]
pub struct Target {
    pub scripts: Box<[Script]>,
    pub is_stage: bool,
    pub name: String,
    pub layer_order: u32,
    pub costumes: Box<[Costume]>,
}
