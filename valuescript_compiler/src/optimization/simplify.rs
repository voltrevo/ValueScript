use std::{collections::HashMap, mem::take};

use crate::asm::{DefinitionContent, FnLine, Function, Instruction, Module, Pointer, Register};

use super::kal::{FnState, Kal};

pub fn simplify(module: &mut Module) {
  let mut pointer_kals = HashMap::<Pointer, Kal>::new();

  for defn in &mut module.definitions {
    if let DefinitionContent::Value(value) = &defn.content {
      pointer_kals.insert(defn.pointer.clone(), Kal::from_value(value));
    }
  }

  for defn in &mut module.definitions {
    match &mut defn.content {
      DefinitionContent::Function(fn_) => simplify_fn(FnState::new(pointer_kals.clone()), fn_),
      DefinitionContent::Class(_) => {}
      DefinitionContent::Value(_) => {}
      DefinitionContent::Lazy(_) => {}
    }
  }
}

fn handle_mutation_releases(body: &mut Vec<FnLine>, i: usize) {
  let mut calls = Vec::<(Register, usize)>::new();

  match &mut body[i] {
    FnLine::Instruction(instr) => {
      let mut skips_needed = 0;

      instr.visit_registers_mut_rev(&mut |rvm| {
        skips_needed += 1;

        if rvm.write && !rvm.read {
          calls.push((rvm.register.clone(), skips_needed));
        }
      });
    }
    FnLine::Release(_) | FnLine::Label(_) | FnLine::Empty | FnLine::Comment(_) => {}
  };

  for (released_reg, skips) in calls {
    handle_release(body, i, released_reg.clone(), skips);
  }
}

fn handle_release(
  body: &mut Vec<FnLine>,
  i: usize,
  released_reg: Register,
  skips_needed: usize,
) -> bool {
  let mut j = i + 1;
  let mut skips = 0;
  let mut taken = false;
  while j > 0 {
    j -= 1;

    let instr = match &mut body[j] {
      FnLine::Instruction(instr) => instr,
      FnLine::Label(_) => return false,
      _ => continue,
    };

    if is_jmp_instr(instr) {
      return false;
    }

    let mut write_found = false;

    if !taken {
      instr.visit_registers_mut_rev(&mut |rvm| {
        if skips < skips_needed {
          skips += 1;
          return;
        }

        if rvm.register.name != released_reg.name {
          return;
        }

        if !taken && rvm.read && !rvm.write {
          *rvm.register = rvm.register.take();
          taken = true;
        }

        if !write_found && rvm.write {
          write_found = true;

          if !rvm.read && !taken {
            *rvm.register = Register::ignore();
          }
        }
      });
    }

    if write_found {
      break;
    }
  }

  taken
}

fn simplify_fn(mut state: FnState, fn_: &mut Function) {
  let mut pending_releases = Vec::<Register>::new();

  for i in 0..fn_.body.len() {
    if let FnLine::Instruction(Instruction::RequireMutableThis) = &fn_.body[i] {
      if state.mutable_this_established {
        fn_.body[i] = FnLine::Comment(fn_.body[i].to_string());
        continue;
      }
    }

    if is_jmp_or_label(&fn_.body[i]) && i > 0 {
      for released_reg in take(&mut pending_releases) {
        handle_release(&mut fn_.body, i - 1, released_reg, 0);
      }
    }

    match &mut fn_.body[i] {
      FnLine::Instruction(instr) => state.eval_instruction(instr),
      FnLine::Label(_) => state.clear_local(),
      FnLine::Empty | FnLine::Comment(_) => {}
      FnLine::Release(reg) => pending_releases.push(reg.clone()),
    }

    handle_mutation_releases(&mut fn_.body, i);
  }

  if !fn_.body.is_empty() {
    let last_i = fn_.body.len() - 1;

    for released_reg in pending_releases {
      handle_release(&mut fn_.body, last_i, released_reg, 0);
    }
  }
}

fn is_jmp_or_label(line: &FnLine) -> bool {
  match line {
    FnLine::Instruction(instr) => is_jmp_instr(instr),
    FnLine::Label(_) => true,
    FnLine::Empty | FnLine::Comment(_) | FnLine::Release(_) => false,
  }
}

fn is_jmp_instr(instr: &Instruction) -> bool {
  match instr {
    Instruction::End
    | Instruction::Jmp(..)
    | Instruction::JmpIf(..)
    | Instruction::JmpIfNot(..) => true,
    Instruction::Mov(..)
    | Instruction::OpInc(..)
    | Instruction::OpDec(..)
    | Instruction::OpPlus(..)
    | Instruction::OpMinus(..)
    | Instruction::OpMul(..)
    | Instruction::OpDiv(..)
    | Instruction::OpMod(..)
    | Instruction::OpExp(..)
    | Instruction::OpEq(..)
    | Instruction::OpNe(..)
    | Instruction::OpTripleEq(..)
    | Instruction::OpTripleNe(..)
    | Instruction::OpAnd(..)
    | Instruction::OpOr(..)
    | Instruction::OpNot(..)
    | Instruction::OpLess(..)
    | Instruction::OpLessEq(..)
    | Instruction::OpGreater(..)
    | Instruction::OpGreaterEq(..)
    | Instruction::OpNullishCoalesce(..)
    | Instruction::OpOptionalChain(..)
    | Instruction::OpBitAnd(..)
    | Instruction::OpBitOr(..)
    | Instruction::OpBitNot(..)
    | Instruction::OpBitXor(..)
    | Instruction::OpLeftShift(..)
    | Instruction::OpRightShift(..)
    | Instruction::OpRightShiftUnsigned(..)
    | Instruction::TypeOf(..)
    | Instruction::InstanceOf(..)
    | Instruction::In(..)
    | Instruction::Call(..)
    | Instruction::Apply(..)
    | Instruction::Bind(..)
    | Instruction::Sub(..)
    | Instruction::SubMov(..)
    | Instruction::SubCall(..)
    | Instruction::UnaryPlus(..)
    | Instruction::UnaryMinus(..)
    | Instruction::New(..)
    | Instruction::Throw(..)
    | Instruction::Import(..)
    | Instruction::ImportStar(..)
    | Instruction::SetCatch(..)
    | Instruction::UnsetCatch
    | Instruction::ConstSubCall(..)
    | Instruction::RequireMutableThis
    | Instruction::ThisSubCall(..)
    | Instruction::Next(..)
    | Instruction::UnpackIterRes(..)
    | Instruction::Cat(..)
    | Instruction::Yield(..)
    | Instruction::YieldStar(..) => false,
  }
}
