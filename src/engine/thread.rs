use crate::engine::instruction::Script;
use crate::scalar_value::ScalarValue;

use super::trigger::Trigger;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ThreadStatus {
    Running,
    Yield,
    YieldTick,
    Done,
}

#[derive(Debug)]
pub struct StackFrame {
    pub frame_value: ScalarValue,
}

pub type Stack = Vec<ScalarValue>;

#[derive(Debug)]
pub struct Thread<'a> {
    pub code: &'a Script,
    stack: Vec<ScalarValue>,
    stack_frames: Vec<StackFrame>,
    pub instruction_pointer: usize,
    pub status: ThreadStatus,
}

impl<'a> Thread<'a> {
    pub fn new(code: &'a Script) -> Self {
        Thread {
            code,
            stack: vec![],
            stack_frames: vec![],
            instruction_pointer: 0,
            status: ThreadStatus::Done,
        }
    }

    pub fn trigger_matches(&self, trigger: &Trigger) -> bool {
        self.code.trigger == *trigger
    }

    pub fn start(&mut self) {
        self.status = ThreadStatus::Running;
        self.instruction_pointer = 0;
        self.stack.clear();
        self.stack_frames.clear();
    }

    pub fn resume(&mut self) {
        self.status = ThreadStatus::Running;
    }

    pub fn yield_thread(&mut self) {
        self.status = ThreadStatus::Yield;
    }

    pub fn push_stack(&mut self, value: ScalarValue) {
        self.stack.push(value);
    }

    pub fn pop_stack(&mut self) -> ScalarValue {
        self.stack.pop().unwrap()
    }

    pub fn peek_stack(&mut self) -> &ScalarValue {
        self.stack.last().unwrap()
    }

    pub fn push_stack_frame(&mut self) {
        self.stack_frames.push(StackFrame {
            frame_value: ScalarValue::Num(0.0),
        })
    }

    pub fn pop_stack_frame(&mut self) -> StackFrame {
        self.stack_frames.pop().unwrap()
    }

    pub fn peek_stack_frame(&mut self) -> &mut StackFrame {
        self.stack_frames.last_mut().unwrap()
    }

    pub fn write_frame_value(&mut self) {
        // TODO: lots of duplicated code here because the helper functions must
        // borrow the entire struct. There's gotta be a better way to do this
        self.stack.push(self.stack.last().unwrap().clone())
    }
}
