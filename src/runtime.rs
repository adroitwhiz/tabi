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
    project: &'a Project,
    exec_contexts: Vec<ExecutionContext<'a>>,
    redraw_requested: bool,
}

pub struct ExecutionContext<'a> {
    pub sprite: Sprite<'a>,
    pub threads: Vec<Thread<'a>>,
}

const STEP_TIME: Duration = Duration::from_nanos(33333333);

impl<'a, 'eng> Runtime<'a, 'eng> {
    pub fn new(project: &'a Project, engine_data: &'eng EngineData) -> Self {
        let mut rt = Runtime {
            engine_data,
            exec_contexts: Vec::new(),
            project,
            redraw_requested: false,
        };

        let exec_contexts = &mut rt.exec_contexts;

        rt.project.targets.iter().for_each(|target| {
            let threads: Vec<Thread> = target
                .scripts
                .iter()
                .map(|script| Thread::new(script))
                .collect();
            let sprite = Sprite::new(target);
            exec_contexts.push(ExecutionContext { sprite, threads });
        });

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
