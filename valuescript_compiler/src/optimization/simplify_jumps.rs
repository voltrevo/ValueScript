use std::collections::HashMap;

use crate::{
  asm::{DefinitionContent, FnLine, Function, Module, Structured},
  instruction::Instruction,
};

pub fn simplify_jumps(module: &mut Module) {
  for defn in &mut module.definitions {
    if let DefinitionContent::Function(fn_) = &mut defn.content {
      simplify_jumps_fn(fn_);
    }
  }
}

fn simplify_jumps_fn(fn_: &mut Function) {
  let mut substitutions = HashMap::<usize, FnLine>::new();

  for i in 0..fn_.body.len() {
    if let FnLine::Instruction(Instruction::End) = &fn_.body[i] {
      if next_instruction_index(&fn_.body, i).is_none() {
        // Remove `end` instructions when we're already at the end of the function.
        substitutions.insert(i, FnLine::Comment(Structured(&fn_.body[i]).to_string()));
        continue;
      }
    }

    let (conditional, label_ref) = match &fn_.body[i] {
      FnLine::Instruction(instr) => match instr {
        Instruction::Jmp(label_ref) => (false, label_ref),
        Instruction::JmpIf(_, label_ref) => (true, label_ref),
        Instruction::JmpIfNot(_, label_ref) => (true, label_ref),
        _ => continue,
      },
      _ => continue,
    };

    let i_next_instr = next_instruction_index(&fn_.body, i);

    let mut j = 0;

    // Find matching label
    while j < fn_.body.len() {
      if let FnLine::Label(label) = &fn_.body[j] {
        if label.name == label_ref.name {
          break;
        }
      }

      j += 1;
    }

    let j_next_instr = next_instruction_index(&fn_.body, j);

    if i_next_instr == j_next_instr {
      // The next instruction is the same regardless of whether we jump. So don't jump.
      substitutions.insert(i, FnLine::Comment(Structured(&fn_.body[i]).to_string()));
    }

    if !conditional {
      match j_next_instr {
        Some(j_next_instr) => match &fn_.body[j_next_instr] {
          FnLine::Instruction(Instruction::End) => {
            // Instead of jumping to `end`, just end
            substitutions.insert(i, FnLine::Instruction(Instruction::End));
          }
          FnLine::Instruction(Instruction::Jmp(label_ref)) => {
            // Instead of jumping to another jump, jump directly to where that one goes
            substitutions.insert(i, FnLine::Instruction(Instruction::Jmp(label_ref.clone())));
          }
          FnLine::Instruction(_) => {}
          FnLine::Label(_) | FnLine::Empty | FnLine::Comment(_) | FnLine::Release(_) => {
            panic!("Jump to non-instruction")
          }
        },
        None => {
          // None means that the jump goes to the end of the function, so just end.
          substitutions.insert(i, FnLine::Instruction(Instruction::End));
        }
      }
    }
  }

  for (i, line) in &mut fn_.body.iter_mut().enumerate() {
    if let Some(substitution) = substitutions.get_mut(&i) {
      *line = substitution.clone();
    }
  }
}

fn next_instruction_index(body: &Vec<FnLine>, mut i: usize) -> Option<usize> {
  i += 1;

  while i < body.len() {
    match &body[i] {
      FnLine::Instruction(_) => return Some(i),
      FnLine::Label(_) | FnLine::Empty | FnLine::Comment(_) | FnLine::Release(_) => {}
    }

    i += 1;
  }

  None
}
