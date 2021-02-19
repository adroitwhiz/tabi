use crate::{
    blocks::block::{Block, BlockInput},
    engine::instruction::{Instruction, Script},
    engine::trigger::Trigger,
    scalar_value::ScalarValue,
};

pub fn compile_block_input(
    input: &BlockInput,
    instructions: &mut Vec<Instruction>,
    blocks: &Vec<Block>,
) {
    match input {
        BlockInput::Literal(v) => instructions.push(Instruction::Push(v.clone())),
        BlockInput::Reporter(block) => compile_block(block, instructions, blocks),
        BlockInput::Substack(idx) => compile_substack(*idx, instructions, blocks),
    }
}

pub fn compile_block(block: &Block, instructions: &mut Vec<Instruction>, blocks: &Vec<Block>) {
    match block.spec.name {
        "math_number" => {
            compile_block_input(&block.field_values[0], instructions, blocks);
        }
        "motion_movesteps" => {
            compile_block_input(&block.field_values[0], instructions, blocks);
            instructions.push(Instruction::MoveSteps);
        }
        "control_repeat" => {
            instructions.push(Instruction::SaveStackFrame);
            compile_block_input(&block.field_values[0 /* TIMES */], instructions, blocks);
            instructions.push(Instruction::WriteFrameValue);

            // Run at the start of each iteration
            let label_iteration_start = instructions.len();
            instructions.push(Instruction::ReadFrameValue);
            instructions.push(Instruction::Push(ScalarValue::Num(0.5)));
            instructions.push(Instruction::LessThan);

            // BACKPATCH THIS to the cleanup code
            let jump_to_cleanup = instructions.len();
            instructions.push(Instruction::JumpIfTrue(0));

            instructions.push(Instruction::ReadFrameValue);
            instructions.push(Instruction::Push(ScalarValue::Num(1.0)));
            instructions.push(Instruction::Subtract);
            instructions.push(Instruction::WriteFrameValue);
            compile_block_input(
                &block.field_values[1], /* SUBSTACK */
                instructions,
                blocks,
            );

            instructions.push(Instruction::Jump(label_iteration_start));

            let label_cleanup_code = instructions.len();
            instructions.push(Instruction::RestoreStackFrame);

            instructions[jump_to_cleanup] = Instruction::JumpIfTrue(label_cleanup_code);
        }

        _ => {
            println!("Unknown opcode {}", block.spec.name);
        }
    }
}

pub fn compile_substack(
    substack_id: usize,
    instructions: &mut Vec<Instruction>,
    blocks: &Vec<Block>,
) {
    let mut next_idx = Some(substack_id);

    while let Some(block_idx) = next_idx {
        let block = &blocks[block_idx];
        next_idx = block.next;

        compile_block(block, instructions, blocks);
    }
}

pub fn compile_hat(block: &Block) -> Trigger {
    match block.spec.name {
        "event_whenflagclicked" => Trigger::WhenFlagClicked,
        _ => panic!("Unknown hat opcode {}", block.spec.name),
    }
}

pub fn compile_blocks(blocks: &Vec<Block>) -> Vec<Script> {
    let mut scripts = Vec::new();

    blocks
        .into_iter()
        .filter(|block| block.parent == None)
        .for_each(|root_block| {
            let mut instructions: Vec<Instruction> = Vec::new();

            // TODO: this skips the first block but expressions in edge-triggered hats must be compiled too
            compile_substack(root_block.next.unwrap(), &mut instructions, blocks);

            scripts.push(Script {
                instructions,
                trigger: compile_hat(root_block),
            })
        });

    scripts
}
