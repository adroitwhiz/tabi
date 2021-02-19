pub mod scalar_value;

pub mod engine {
    pub mod engine_data;
    pub mod execute;
    pub mod instruction;
    pub mod project;
    pub mod sprite;
    pub mod target;
    pub mod thread;
    pub mod trigger;
}

pub mod blocks {
    pub mod block;
    pub mod block_specs;
}

pub mod compile;
pub mod deserialize;
pub mod runtime;

use crate::engine::engine_data::EngineData;

use std::{fs, io::Read};
use zip;

fn main() {
    /*let t = target::Target {
        scripts: vec![],
        is_stage: false
    };
    let s = sprite::Sprite::new(&t);*/

    let eng_data = EngineData::new();

    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    let fname = std::path::Path::new(&*args[1]);
    let file = fs::File::open(&fname).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    let mut project_json = archive.by_name("project.json").unwrap();
    let mut buffer = String::new();
    project_json.read_to_string(&mut buffer).unwrap();

    let d = deserialize::deserialize_project(&buffer, &eng_data);

    match d {
        Ok(p) => {
            println!("{:?}", p);
        }
        Err(e) => {
            println!("{}", e);
        }
    }

    println!("{:?}", eng_data.block_specs);

    std::process::exit(0);
}
