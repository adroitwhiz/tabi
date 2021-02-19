use crate::blocks::block_specs;

pub struct EngineData {
    pub block_specs: block_specs::BlockSpecMap,
    pub strings: Vec<String>,
}

impl EngineData {
    pub fn new() -> Self {
        EngineData {
            block_specs: block_specs::make_block_specs(),
            strings: vec![],
        }
    }
}
