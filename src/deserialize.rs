use crate::{
    blocks::{
        block::{Block, BlockInput},
        block_specs::BlockSpecMap,
    },
    compile::compile_blocks,
    data::asset,
    engine::{engine_data::EngineData, project, target},
    scalar_value::ScalarValue,
};

use std::convert::TryFrom;
use std::{collections::HashMap, convert::TryInto, fs::File, io::Read};

use md5::Digest;
use serde_json::{Map, Value};

use num_enum::TryFromPrimitive;
use zip::ZipArchive;

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

fn deserialize_asset(
    serialized_asset: &Map<String, Value>,
    archive: &mut ZipArchive<File>,
) -> Result<asset::Asset, &'static str> {
    let md5ext = serialized_asset["md5ext"]
        .as_str()
        .ok_or_else(|| "asset has no md5ext")?;
    let md5_str = serialized_asset["assetId"]
        .as_str()
        .ok_or_else(|| "asset has no assetId")?;
    let asset_type_str = serialized_asset["dataFormat"]
        .as_str()
        .ok_or_else(|| "asset has no dataFormat")?;

    let mut asset_file = archive
        .by_name(md5ext)
        .map_err(|_| "asset not found in zip")?;
    let mut asset_data: Vec<u8> = Vec::with_capacity(asset_file.size() as usize);
    asset_file
        .read_to_end(&mut asset_data)
        .map_err(|_| "could not read asset file")?;

    let md5_bytes = hex::decode(md5_str).map_err(|_| "could not decode assetId")?;

    let asset_type = match asset_type_str {
        "png" => Ok(asset::AssetType::PNG),
        "svg" => Ok(asset::AssetType::SVG),
        "jpg" => Ok(asset::AssetType::JPEG),
        "mp3" => Ok(asset::AssetType::MP3),
        "wav" => Ok(asset::AssetType::WAV),
        _ => Err("unknown asset type"),
    }?;

    Ok(asset::Asset {
        data: asset_data.into_boxed_slice(),
        md5_digest: Digest(
            TryInto::<[u8; 16]>::try_into(md5_bytes).map_err(|_| "could not decode assetId")?,
        ),
        asset_type,
    })
}

fn deserialize_costume(
    serialized_costume: &Map<String, Value>,
    archive: &mut ZipArchive<File>,
) -> Result<asset::Costume, &'static str> {
    let d_asset = deserialize_asset(serialized_costume, archive)?;
    let rotation_center_x = serialized_costume["rotationCenterX"]
        .as_f64()
        .ok_or_else(|| "costume has no rotationCenterX")?;
    let rotation_center_y = serialized_costume["rotationCenterY"]
        .as_f64()
        .ok_or_else(|| "costume has no rotationCenterY")?;
    let name = serialized_costume["name"]
        .as_str()
        .ok_or_else(|| "costume has no name")?;

    Ok(asset::Costume {
        asset: d_asset,
        rotation_center: (rotation_center_x, rotation_center_y),
        name: name.to_string(),
    })
}

fn deserialize_target<'eng>(
    serialized_target: &Map<String, Value>,
    archive: &mut ZipArchive<File>,
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
    let costumes = serialized_target["costumes"]
        .as_array()
        .ok_or_else(|| "target has no costumes")?;
    let d_blocks = deserialize_blocks(blocks, eng_data)?;
    let mut d_costumes = Vec::with_capacity(costumes.len());
    for costume in costumes {
        d_costumes.push(deserialize_costume(
            costume
                .as_object()
                .ok_or_else(|| "costume is not an object")?,
            archive,
        )?);
    }
    println!("{:#?}", d_blocks);
    Ok(target::Target {
        scripts: compile_blocks(&d_blocks),
        is_stage,
        name: name.to_string(),
        layer_order: layer_order as u32,
        costumes: d_costumes.into_boxed_slice(),
    })
}

pub fn deserialize_project<'a, 'eng>(
    archive: &mut ZipArchive<File>,
    eng_data: &'eng EngineData,
) -> Result<project::Project, &'a str> {
    let mut json = String::new();
    {
        let mut project_json_file = archive
            .by_name("project.json")
            .map_err(|_| "project.json not found")?;
        project_json_file
            .read_to_string(&mut json)
            .map_err(|_| "Could not read project.json")?;
    }

    let v: Value = serde_json::from_str(&json).map_err(|_| "Could not deserialize JSON")?;

    let mut targets = vec![];

    if let serde_json::Value::Array(serialized_targets) = &v["targets"] {
        serialized_targets
            .into_iter()
            .try_for_each(|target| -> Result<(), &str> {
                if let serde_json::Value::Object(target) = target {
                    match deserialize_target(&target, archive, &eng_data) {
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
