use crate::asm::{
  Definition, DefinitionContent, Instruction, InstructionOrLabel, Pointer, Register, Value,
};

pub struct ImportPattern {
  pub pointer: Pointer,
  pub path: String,
  pub kind: ImportKind,
}

pub enum ImportKind {
  Default,
  Star,
  Name(String),
}

impl ImportPattern {
  pub fn decode(definition: &Definition) -> Option<ImportPattern> {
    let lazy = match &definition.content {
      DefinitionContent::Lazy(lazy) => lazy,
      _ => return None,
    };

    if lazy.body.len() > 2 {
      return None;
    }

    let first_instruction = match lazy.body.first() {
      Some(InstructionOrLabel::Instruction(instruction)) => instruction,
      _ => return None,
    };

    let (path_value, is_star) = match first_instruction {
      Instruction::Import(path, Register::Return) => (path, false),
      Instruction::ImportStar(path, Register::Return) => (path, true),
      _ => return None,
    };

    let path = match path_value {
      Value::String(path) => path,
      _ => return None,
    };

    let second_instruction_opt = lazy.body.get(1);

    if !is_star {
      return match second_instruction_opt {
        Some(_) => None,
        _ => Some(ImportPattern {
          pointer: definition.pointer.clone(),
          path: path.clone(),
          kind: ImportKind::Default,
        }),
      };
    }

    let second_instruction = match second_instruction_opt {
      Some(InstructionOrLabel::Instruction(instruction)) => instruction,
      Some(_) => return None,
      _ => {
        return Some(ImportPattern {
          pointer: definition.pointer.clone(),
          path: path.clone(),
          kind: ImportKind::Star,
        })
      }
    };

    match second_instruction {
      Instruction::Sub(
        Value::Register(Register::Return),
        Value::String(name),
        Register::Return,
      ) => Some(ImportPattern {
        pointer: definition.pointer.clone(),
        path: path.clone(),
        kind: ImportKind::Name(name.clone()),
      }),
      _ => None,
    }
  }
}
