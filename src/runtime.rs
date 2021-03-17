use crate::engine::{
    engine_data::EngineData,
    execute::execute,
    project::Project,
    sprite::Sprite,
    thread::{Thread, ThreadStatus},
    trigger::Trigger,
};

use std::time::{Duration, Instant};

pub struct Runtime<'a, 'eng> {
    engine_data: &'eng EngineData,
    threads: Vec<Thread<'a>>,
    project: &'a Project,
    sprites: Vec<Sprite<'a>>,
    redraw_requested: bool,
}

const STEP_TIME: Duration = Duration::from_nanos(33333333);

impl<'a, 'eng> Runtime<'a, 'eng> {
    pub fn new(project: &'a Project, engine_data: &'eng EngineData) -> Self {
        let mut rt = Runtime {
            engine_data,
            threads: Vec::new(),
            project,
            sprites: Vec::new(),
            redraw_requested: false,
        };

        let sprites = &mut rt.sprites;
        let threads = &mut rt.threads;

        rt.project.targets.iter().for_each(|target| {
            let mut thread_indices = Vec::with_capacity(target.scripts.len());
            target.scripts.iter().for_each(|script| {
                threads.push(Thread::new(script));
                thread_indices.push(threads.len() - 1);
            });
            let sprite = Sprite::new(target, thread_indices.into_boxed_slice());
            sprites.push(sprite);
        });

        rt.sprites
            .sort_by_cached_key(|sprite| sprite.target.layer_order);

        rt
    }

    pub fn start_hats(&mut self, trigger: &Trigger) {
        for thread in &mut self.threads {
            // TODO: only some trigger types restart running threads.
            if thread.trigger_matches(trigger) {
                thread.start()
            }
        }
    }

    fn step_threads(&mut self) {
        let start_time = Instant::now();
        let mut ran_first_tick = false;

        loop {
            let mut num_active_threads = 0;

            for sprite in &mut self.sprites {
                for thread_idx in sprite.thread_indices.clone().iter() {
                    let thread = &mut self.threads[*thread_idx];
                    let thread_status = thread.status;

                    if thread_status == ThreadStatus::Done {
                        break;
                    }

                    if thread_status == ThreadStatus::YieldTick && !ran_first_tick {
                        thread.resume();
                    }

                    if thread_status == ThreadStatus::Running
                        || thread_status == ThreadStatus::Yield
                    {
                        execute(sprite, thread);
                    }

                    if thread_status == ThreadStatus::Running {
                        num_active_threads += 1;
                    }
                }
            }

            ran_first_tick = true;

            if num_active_threads == 0
                || Instant::now().duration_since(start_time) >= STEP_TIME
                || self.redraw_requested
            {
                break;
            }
        }
    }
}
