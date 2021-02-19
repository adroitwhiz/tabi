use std::collections::HashMap;

use crate::blocks::block::{BlockFieldType, BlockShape, BlockSpec};

pub type BlockSpecMap = HashMap<String, BlockSpec>;

pub fn make_block_specs() -> BlockSpecMap {
    let mut specs: BlockSpecMap = HashMap::new();

    specs.insert(
        "math_number".to_string(),
        BlockSpec {
            name: "math_number",
            field_names: Box::new(["NUM".to_string()]),
            field_types: Box::new([BlockFieldType::Field]),
            shape: BlockShape::Reporter,
        },
    );

    specs.insert(
        "motion_movesteps".to_string(),
        BlockSpec {
            name: "motion_movesteps",
            field_names: Box::new(["STEPS".to_string()]),
            field_types: Box::new([BlockFieldType::Input]),
            shape: BlockShape::Command,
        },
    );

    specs.insert(
        "control_repeat".to_string(),
        BlockSpec {
            name: "control_repeat",
            field_names: Box::new(["TIMES".to_string(), "SUBSTACK".to_string()]),
            field_types: Box::new([BlockFieldType::Input, BlockFieldType::Input]),
            shape: BlockShape::Command,
        },
    );

    specs.insert(
        "event_whenflagclicked".to_string(),
        BlockSpec {
            name: "event_whenflagclicked",
            field_names: Box::new([]),
            field_types: Box::new([]),
            shape: BlockShape::Hat,
        },
    );

    specs
}
