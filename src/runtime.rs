use crate::engine::{engine_data::EngineData, project::Project, sprite::Sprite, execute::execute, thread::{Thread, ThreadStatus}, trigger::Trigger};

use std::{cell::RefCell, rc::Rc, time::{Duration, Instant}};

struct ThreadIndex {
    pub sprite_idx: u32,
    pub thread_idx: u32,
}

pub struct Runtime<'a, 'eng> {
    engine_data: &'eng EngineData,
    threads: Vec<ThreadIndex>,
    project: &'a Project,
    sprites: Vec<Rc<RefCell<Sprite<'a>>>>,
    redraw_requested: bool,
}

pub struct ExecutionContext<'a> {
    pub current_thread: &'a mut Thread<'a>,
    pub current_sprite: RefCell<Sprite<'a>>
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
            let sprite = Sprite::new(target);
            sprites.push(Rc::new(RefCell::new(sprite)));
            let sprite_threads = &sprites.last().unwrap().borrow().threads;

            sprite_threads
                .iter()
                .enumerate()
                .for_each(|(thread_idx, _)| {
                    threads.push(ThreadIndex {
                        sprite_idx: (sprites.len() - 1) as u32,
                        thread_idx: thread_idx as u32,
                    });
                })
        });

        rt.sprites
            .sort_by_cached_key(|sprite| sprite.borrow().target.layer_order);

        rt
    }

    pub fn start_hats(&mut self, trigger: &Trigger) {
        for idx in &mut self.threads {
            let sprite = self.sprites[idx.sprite_idx as usize].borrow();
            let mut thread = sprite.threads[idx.thread_idx as usize].borrow_mut();
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

            for idx in &self.threads {
                // TODO: find a way to deduplicate this code
                let current_sprite_ref = &self.sprites[idx.sprite_idx as usize];
                let current_sprite =  current_sprite_ref.borrow_mut();
                let mut thread =
                    current_sprite.threads[idx.thread_idx as usize].borrow_mut();

                if thread.status == ThreadStatus::Done {
                    break;
                }

                if thread.status == ThreadStatus::YieldTick && !ran_first_tick {
                    thread.resume();
                }

                if thread.status == ThreadStatus::Running || thread.status == ThreadStatus::Yield {
                    execute(current_sprite_ref.clone(), idx.thread_idx as usize);
                }

                if thread.status == ThreadStatus::Running {
                    num_active_threads += 1;
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
