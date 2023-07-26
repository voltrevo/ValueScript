use crate::asm::{Definition, DefinitionContent, FnLine, Instruction, Pointer, Value};

pub struct ImportPattern {
  pub pointer: Pointer,
  pub path: String,
  pub kind: ImportKind,
}

#[derive(PartialEq, Eq)]
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
      Some(FnLine::Instruction(instruction)) => instruction,
      _ => return None,
    };

    let (path_value, is_star) = match first_instruction {
      Instruction::Import(path, reg) => match reg.name.as_str() {
        "return" => (path, false),
        _ => return None,
      },
      Instruction::ImportStar(path, reg) => match reg.name.as_str() {
        "return" => (path, true),
        _ => return None,
      },
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
      Some(FnLine::Instruction(instruction)) => instruction,
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
      Instruction::Sub(Value::Register(obj), Value::String(name), target) => {
        match obj.name == "return" && target.name == "return" {
          true => Some(ImportPattern {
            pointer: definition.pointer.clone(),
            path: path.clone(),
            kind: ImportKind::Name(name.clone()),
          }),
          false => None,
        }
      }
      _ => None,
    }
  }
}
