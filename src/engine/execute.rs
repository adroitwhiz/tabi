use std::{cell::RefCell, cmp::Ordering, rc::Rc};

use crate::{scalar_value::ScalarValue};

use super::{instruction::Instruction, sprite::Sprite};

pub fn execute(sprite: Rc<RefCell<Sprite>>, thread_idx: usize) {
    let sprite_ref = sprite.borrow_mut();
    let mut current_thread = sprite_ref.threads[thread_idx].borrow_mut();
    let instruction = &current_thread.code.instructions[current_thread.instruction_pointer];

    let mut did_jump = false;
    match instruction {
        // Internal
        Instruction::Push(value) => {
            current_thread.push_stack(value.clone())
        },

        Instruction::Yield => {
            current_thread.yield_thread()
        },

        Instruction::Jump(to) => {
            current_thread.instruction_pointer = *to;
            did_jump = true;
        },

        Instruction::JumpIfTrue(to) => {
            if bool::from(current_thread.pop_stack()) {
                current_thread.instruction_pointer = *to;
                did_jump = true;
            }
        },

        Instruction::SaveStackFrame => {
            current_thread.push_stack_frame();
        },

        Instruction::RestoreStackFrame => {
            current_thread.pop_stack_frame();
        },

        Instruction::ReadFrameValue => {
            let v = current_thread.peek_stack_frame().frame_value.clone();
            current_thread.push_stack(v);
        },

        Instruction::WriteFrameValue => {
            current_thread.write_frame_value();
        },

        Instruction::RequestRedraw => {
            unimplemented!();
        }

        // Operators
        Instruction::Add => {
            let op1 = current_thread.pop_stack();
            let op2 = current_thread.pop_stack();
            current_thread.push_stack(ScalarValue::Num(f64::from(op2) + f64::from(op1)));
        },

        Instruction::Subtract => {
            let op1 = current_thread.pop_stack();
            let op2 = current_thread.pop_stack();
            current_thread.push_stack(ScalarValue::Num(f64::from(op2) - f64::from(op1)));
        },

        Instruction::LessThan => {
            let op1 = current_thread.pop_stack();
            let op2 = current_thread.pop_stack();
            current_thread.push_stack(ScalarValue::Bool(op2.compare(&op1) == Ordering::Less));
        },

        Instruction::Equals => {
            let op1 = current_thread.pop_stack();
            let op2 = current_thread.pop_stack();
            current_thread.push_stack(ScalarValue::Bool(op2.compare(&op1) == Ordering::Equal));
        },

        Instruction::GreaterThan => {
            let op1 = current_thread.pop_stack();
            let op2 = current_thread.pop_stack();
            current_thread.push_stack(ScalarValue::Bool(op2.compare(&op1) == Ordering::Greater));
        },

        // Motion
        Instruction::GotoXY => {
            let op1 = current_thread.pop_stack();
            let op2 = current_thread.pop_stack();
            sprite.clone().borrow_mut().move_to(f64::from(op1), f64::from(op2));
        },

        Instruction::MoveSteps => {
            let steps = f64::from(current_thread.pop_stack());
            let angle = (90.0 - sprite_ref.direction) * (std::f64::consts::PI / 180.0);
            sprite.clone().borrow_mut().move_to(
                sprite_ref.x + (angle.sin() * steps),
                sprite_ref.y + (angle.cos() * steps)
            );
        }
    }

    if !did_jump {
        current_thread.instruction_pointer += 1;
    }
}
