pub mod scalar_value;

pub mod engine {
    pub mod costume;
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

pub mod data {
    pub mod asset;
}

pub mod compile;
pub mod deserialize;
pub mod runtime;
pub mod renderer {
    pub mod bitmap_skin;
    pub mod blank_skin;
    pub mod common;
    pub mod drawable;
    pub mod renderer;
    pub mod skin;
    pub mod svg_skin;
}

use crate::engine::{engine_data::EngineData, trigger::Trigger};

use renderer::renderer::Renderer;
use runtime::Runtime;
use std::{cell::RefCell, error::Error, fs, time::{Duration, Instant}};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::Window,
};
use zip;

fn run() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let eng_data = EngineData::new();

    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    let fname = std::path::Path::new(&args[1]);
    let file = fs::File::open(&fname).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    println!("{:?}", eng_data.block_specs);

    let mut event_loop = EventLoop::new();
    let window = Window::new(&event_loop)?;
    window.set_title("Tabi");
    window.set_inner_size(winit::dpi::LogicalSize {
        width: 480,
        height: 360,
    });
    let size = window.inner_size();

    let renderer = RefCell::new(Renderer::with_window(&window, (size.width, size.height), (480, 360)));

    let project = deserialize::deserialize_project(&mut archive, &eng_data, &mut renderer.borrow_mut())?;

    println!("{:?}", project);

    let mut runtime = Runtime::new(&project, &eng_data, &renderer);

    runtime.start_hats(&Trigger::WhenFlagClicked);

    let mut last_update_inst = Instant::now();

    event_loop.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawEventsCleared => {
                let target_frametime = Duration::from_secs_f64(1.0 / 30.0);
                let time_since_last_frame = last_update_inst.elapsed();
                if time_since_last_frame >= target_frametime {
                    window.request_redraw();
                    last_update_inst = Instant::now();
                } else {
                    *control_flow = ControlFlow::WaitUntil(
                        Instant::now() + target_frametime - time_since_last_frame,
                    );
                }
            }
            Event::RedrawRequested(_) => {
                runtime.step();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => runtime.resize((size.width, size.height)),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });

    Ok(())
}

fn main() {
    run().ok();

    std::process::exit(0);
}
