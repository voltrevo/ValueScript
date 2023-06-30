use std::mem::take;

use crate::asm::{DefinitionContent, FnLine, Module};

pub fn remove_meta_lines(module: &mut Module) {
  for defn in &mut module.definitions {
    if let DefinitionContent::Function(fn_) = &mut defn.content {
      for line in take(&mut fn_.body) {
        match &line {
          FnLine::Instruction(_) | FnLine::Label(_) | FnLine::Empty => fn_.body.push(line),
          FnLine::Comment(_) | FnLine::Release(_) => continue,
        }
      }
    }
  }
}
