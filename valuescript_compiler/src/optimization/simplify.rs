use crate::asm::{DefinitionContent, FnLine, Function, Instruction, Module};

pub fn simplify(module: &mut Module) {
  for defn in &mut module.definitions {
    match &mut defn.content {
      DefinitionContent::Function(fn_) => simplify_fn(FnState::default(), fn_),
      DefinitionContent::Class(_) => {}
      DefinitionContent::Value(_) => {}
      DefinitionContent::Lazy(_) => {}
    }
  }
}

#[derive(Default)]
struct FnState {
  mutable_this_established: bool,
  // registers: HashMap<Register, Value>,
}

impl FnState {
  fn clear(&mut self) {
    *self = Self::default();
  }
}

fn simplify_fn(mut state: FnState, fn_: &mut Function) {
  for line in &mut fn_.body {
    match line {
      FnLine::Instruction(instr) => {
        if let Instruction::RequireMutableThis = instr {
          if state.mutable_this_established {
            *line = FnLine::Empty;
          } else {
            state.mutable_this_established = true;
          }
        }
      }
      FnLine::Label(..) => state.clear(),
      FnLine::Empty | FnLine::Comment(..) => {}
    }
  }
}
