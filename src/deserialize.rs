use crate::{
    blocks::{
        block::{Block, BlockInput},
        block_specs::BlockSpecMap,
    },
    compile::compile_blocks,
    engine::{engine_data::EngineData, project, target},
    scalar_value::ScalarValue,
};

use std::collections::HashMap;
use std::convert::TryFrom;

use serde_json::{Map, Value};

use num_enum::TryFromPrimitive;

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum InputDescriptorShadowStatus {
    UnobscuredShadow = 1,
    NoShadow,
    ObscuredShadow,
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum InputDescriptorPrimitive {
    MathNumber = 4,
    MathPositiveNumber,
    MathWholeNumber,
    MathInteger,
    MathAngle,
    ColourPicker,
    Text,
    EventBroadcastMenu,
    DataVariable,
    DataListcontents,
}

fn deserialize_input_descriptor<'eng>(
    serialized_input_descriptor: &Vec<Value>,
    block_specs: &'eng BlockSpecMap,
    ids_to_indices: &HashMap<&String, usize>,
    parent: usize,
) -> Result<BlockInput<'eng>, &'static str> {
    /*let shadow_status = InputDescriptorShadowStatus::try_from(
        u8::try_from(
            serialized_input_descriptor[0].as_u64().ok_or_else(|| "Malformed input descriptor")?
        ).map_err(|_| "Malformed input descriptor")?
    ).map_err(|_| "Malformed input descriptor")?;*/

    match &serialized_input_descriptor[1] {
        Value::Array(arr) => {
            let input_primitive = InputDescriptorPrimitive::try_from(
                u8::try_from(
                    arr[0]
                        .as_u64()
                        .ok_or_else(|| "Malformed input descriptor")?,
                )
                .map_err(|_| "Malformed input descriptor")?,
            )
            .map_err(|_| "Malformed input descriptor")?;

            Ok(BlockInput::Reporter(match input_primitive {
                InputDescriptorPrimitive::MathNumber
                | InputDescriptorPrimitive::MathPositiveNumber
                | InputDescriptorPrimitive::MathWholeNumber
                | InputDescriptorPrimitive::MathInteger
                | InputDescriptorPrimitive::MathAngle => Block {
                    spec: block_specs
                        .get("math_number")
                        .ok_or_else(|| "Missing math_number block spec")?,
                    field_values: Box::new([BlockInput::Literal(ScalarValue::try_from(&arr[1])?)]),
                    next: None,
                    parent: Some(parent),
                },
                InputDescriptorPrimitive::ColourPicker => Block {
                    spec: block_specs
                        .get("colour_picker")
                        .ok_or_else(|| "Missing colour_picker block spec")?,
                    field_values: Box::new([BlockInput::Literal(ScalarValue::try_from(&arr[1])?)]),
                    next: None,
                    parent: Some(parent),
                },
                InputDescriptorPrimitive::Text | InputDescriptorPrimitive::EventBroadcastMenu => {
                    Block {
                        spec: block_specs
                            .get("text")
                            .ok_or_else(|| "Missing text block spec")?,
                        field_values: Box::new([BlockInput::Literal(ScalarValue::try_from(
                            &arr[1],
                        )?)]),
                        next: None,
                        parent: Some(parent),
                    }
                }
                InputDescriptorPrimitive::DataVariable => Block {
                    spec: block_specs
                        .get("data_variable")
                        .ok_or_else(|| "Missing data_variable block spec")?,
                    field_values: Box::new([BlockInput::Literal(ScalarValue::try_from(&arr[1])?)]),
                    next: None,
                    parent: Some(parent),
                },
                InputDescriptorPrimitive::DataListcontents => Block {
                    spec: block_specs
                        .get("data_listcontents")
                        .ok_or_else(|| "Missing data_listcontents block spec")?,
                    field_values: Box::new([BlockInput::Literal(ScalarValue::try_from(&arr[1])?)]),
                    next: None,
                    parent: Some(parent),
                },
            }))
        }
        Value::String(substack_id) => Ok(BlockInput::Substack(
            *ids_to_indices
                .get(&substack_id)
                .ok_or_else(|| "Referenced nonexistent substack")?,
        )),
        _ => Err("Malformed input descriptor")?,
    }
}

fn deserialize_block<'eng>(
    serialized_block: &Map<String, Value>,
    block_specs: &'eng BlockSpecMap,
    ids_to_indices: &HashMap<&String, usize>,
    block_id: &String,
) -> Result<Block<'eng>, &'static str> {
    if let serde_json::Value::String(opcode) = &serialized_block["opcode"] {
        match block_specs.get(opcode) {
            Some(spec) => {
                let inputs = serialized_block["inputs"]
                    .as_object()
                    .ok_or_else(|| "block has no inputs")?;
                let fields = serialized_block["fields"]
                    .as_object()
                    .ok_or_else(|| "block has no fields")?;
                let next = &serialized_block["next"];
                let parent = &serialized_block["parent"];

                let mut field_values: Vec<BlockInput> = vec![];

                spec.field_names
                    .into_iter()
                    .try_for_each(|name| -> Result<(), &'static str> {
                        let entry = inputs
                            .get(name)
                            .or_else(|| fields.get(name))
                            .ok_or_else(|| "Could not find block input")?;

                        field_values.push(match entry {
                            Value::Bool(_) | Value::Number(_) | Value::String(_) => {
                                BlockInput::Literal(ScalarValue::try_from(entry)?)
                            }
                            Value::Array(arr) => deserialize_input_descriptor(
                                arr,
                                block_specs,
                                ids_to_indices,
                                *ids_to_indices
                                    .get(block_id)
                                    .ok_or_else(|| "Nonexistent block ID")?,
                            )?,
                            _ => Err("Malformed block input")?,
                        });
                        Ok(())
                    })?;

                let b = Block {
                    spec: &spec,
                    field_values: field_values.into_boxed_slice(),
                    next: match next {
                        Value::String(block_id) => Some(
                            *ids_to_indices
                                .get(&block_id)
                                .ok_or_else(|| "Block references invalid block ID")?,
                        ),
                        _ => None,
                    },
                    parent: match parent {
                        Value::String(block_id) => Some(
                            *ids_to_indices
                                .get(&block_id)
                                .ok_or_else(|| "Block references invalid block ID")?,
                        ),
                        _ => None,
                    },
                };

                assert!(b.field_values.len() == spec.field_names.len());

                Ok(b)
            }
            None => Err("Unknown opcode"),
        }
    } else {
        Err("Opcode is not a string")
    }
}

fn deserialize_blocks<'eng>(
    serialized_blocks: &Map<String, Value>,
    eng_data: &'eng EngineData,
) -> Result<Vec<Block<'eng>>, &'static str> {
    let block_specs = &eng_data.block_specs;

    let mut ids_to_indices: HashMap<&String, usize> = HashMap::new();

    let mut idx: usize = 0;
    serialized_blocks.into_iter().for_each(|kv| {
        ids_to_indices.insert(kv.0, idx);
        idx += 1;
    });

    let num_blocks = idx;

    let mut blocks: Vec<Block> = Vec::with_capacity(num_blocks);

    serialized_blocks
        .into_iter()
        .try_for_each(|kv| -> Result<(), &'static str> {
            let serialized_block = kv.1.as_object().ok_or_else(|| "block is not an object")?;

            blocks.push(deserialize_block(
                serialized_block,
                block_specs,
                &ids_to_indices,
                kv.0,
            )?);
            Ok(())
        })?;

    Ok(blocks)
}

fn deserialize_target<'eng>(
    serialized_target: &Map<String, Value>,
    eng_data: &'eng EngineData,
) -> Result<target::Target, &'static str> {
    let is_stage = serialized_target["isStage"]
        .as_bool()
        .ok_or_else(|| "target has no isStage")?;
    let name = serialized_target["name"]
        .as_str()
        .ok_or_else(|| "target has no name")?;
    let blocks = serialized_target["blocks"]
        .as_object()
        .ok_or_else(|| "target has no blocks")?;
    let layer_order = serialized_target["layerOrder"]
        .as_u64()
        .ok_or_else(|| "target has no layerOrder")?;
    let d_blocks = deserialize_blocks(blocks, eng_data)?;
    println!("{:#?}", d_blocks);
    Ok(target::Target {
        scripts: compile_blocks(&d_blocks),
        is_stage,
        name: name.to_string(),
        layer_order: layer_order as u32,
    })
}

pub fn deserialize_project<'a, 'eng>(
    json: &'a String,
    eng_data: &'eng EngineData,
) -> Result<project::Project, &'a str> {
    let v: Result<Value, serde_json::Error> = serde_json::from_str(json);

    // TODO: probably not the best way to convert errors
    if let Err(_) = v {
        return Err("Could not deserialize JSON");
    }
    let v = v.unwrap();

    let mut targets = vec![];

    if let serde_json::Value::Array(serialized_targets) = &v["targets"] {
        serialized_targets
            .into_iter()
            .try_for_each(|target| -> Result<(), &str> {
                if let serde_json::Value::Object(target) = target {
                    match deserialize_target(&target, &eng_data) {
                        Ok(t) => {
                            targets.push(t);
                            Ok(())
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    return Err("Malformed JSON");
                }
            })?
    } else {
        return Err("Malformed JSON");
    }

    return Ok(project::Project { targets });
}
