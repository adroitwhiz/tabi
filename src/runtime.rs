use crate::{
    engine::{
        engine_data::EngineData,
        execute::execute,
        project::Project,
        sprite::Sprite,
        thread::{Thread, ThreadStatus},
        trigger::Trigger,
    },
    renderer::renderer::Renderer,
};

use std::{cell::RefCell, time::{Duration, Instant}};

pub struct Runtime<'a, 'eng, 'r> {
    engine_data: &'eng EngineData,
    project: &'a Project,
    renderer: &'r RefCell<Renderer>,
    exec_contexts: Vec<ExecutionContext<'a, 'r>>,
    redraw_requested: bool,
}

pub struct ExecutionContext<'a, 'r> {
    pub sprite: Sprite<'a, 'r>,
    pub threads: Vec<Thread<'a>>,
}

const STEP_TIME: Duration = Duration::from_nanos(33333333);

impl<'a, 'eng, 'r> Runtime<'a, 'eng, 'r> {
    pub fn new(
        project: &'a Project,
        engine_data: &'eng EngineData,
        renderer: &'r RefCell<Renderer>,
    ) -> Self {
        let mut exec_contexts = Vec::new();

        project.targets.iter().for_each(|target| {
            let threads: Vec<Thread> = target
                .scripts
                .iter()
                .map(|script| Thread::new(script))
                .collect();
            let sprite = Sprite::new(target, renderer);
            exec_contexts.push(ExecutionContext { sprite, threads });
        });

        let mut rt = Runtime {
            engine_data,
            exec_contexts,
            project,
            renderer,
            redraw_requested: false,
        };

        rt.exec_contexts
            .sort_by_cached_key(|ctx| ctx.sprite.target.layer_order);

        rt
    }

    pub fn start_hats(&mut self, trigger: &Trigger) {
        for ExecutionContext { threads, .. } in &mut self.exec_contexts {
            for thread in threads {
                // TODO: only some trigger types restart running threads.
                if thread.trigger_matches(trigger) {
                    thread.start()
                }
            }
        }
    }

    fn step_threads(&mut self) {
        let start_time = Instant::now();
        let mut ran_first_tick = false;

        loop {
            let mut num_active_threads = 0;

            for ExecutionContext { sprite, threads } in &mut self.exec_contexts {
                for thread in threads {
                    if thread.status == ThreadStatus::Done {
                        break;
                    }

                    if thread.status == ThreadStatus::YieldTick && !ran_first_tick {
                        thread.resume();
                    }

                    if thread.status == ThreadStatus::Running
                        || thread.status == ThreadStatus::Yield
                    {
                        execute(sprite, thread);
                    }

                    if thread.status == ThreadStatus::Running {
                        num_active_threads += 1;
                    }

                    if thread.redraw_requested {
                        thread.redraw_requested = false;
                        self.redraw_requested = true;
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

    pub fn step(&mut self) {
        self.step_threads();
        self.renderer.borrow_mut().draw();
    }

    pub fn resize(&mut self, size: (u32, u32)) {
        self.renderer.borrow_mut().resize(size);
    }
}
