use std::{collections::HashSet, mem::take};

use crate::{
  asm::{DefinitionContent, FnLine, Function, Module},
  instruction::InstructionFieldMut,
};

pub fn remove_unused_labels(module: &mut Module) {
  for defn in &mut module.definitions {
    if let DefinitionContent::Function(fn_) = &mut defn.content {
      remove_unused_labels_fn(fn_);
    }
  }
}

fn remove_unused_labels_fn(fn_: &mut Function) {
  let mut used_labels = HashSet::<String>::new();

  for line in &mut fn_.body {
    match line {
      FnLine::Instruction(instr) => {
        instr.visit_fields_mut(&mut |field| match field {
          InstructionFieldMut::LabelRef(label_ref) => {
            used_labels.insert(label_ref.name.clone());
          }
          InstructionFieldMut::Value(_) | InstructionFieldMut::Register(_) => {}
        });
      }
      FnLine::Label(_) | FnLine::Empty | FnLine::Comment(_) | FnLine::Release(_) => {}
    }
  }

  for line in take(&mut fn_.body) {
    match &line {
      FnLine::Label(label) => {
        if !used_labels.contains(&label.name) {
          continue;
        }
      }
      FnLine::Instruction(_) | FnLine::Empty | FnLine::Comment(_) | FnLine::Release(_) => {}
    }

    fn_.body.push(line);
  }
}
