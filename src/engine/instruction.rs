use crate::scalar_value::ScalarValue;

use super::trigger::Trigger;

#[derive(Debug)]
pub struct Script {
    pub trigger: Trigger,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug)]
pub enum Instruction {
    Push(ScalarValue), // Push a value onto the stack
    Yield,             // Yield this thread
    Jump(usize),       // Absolute unconditional jump forward/backward by this amount
    JumpIfTrue(usize), // Absolute jump forward/backward by this amount if the top value on the stack is true
    SaveStackFrame,
    RestoreStackFrame,
    ReadFrameValue,
    WriteFrameValue,
    RequestRedraw,

    Add,
    Subtract,
    LessThan,
    Equals,
    GreaterThan,

    GotoXY,
    MoveSteps,
}
