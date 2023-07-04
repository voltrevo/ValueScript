use std::collections::{HashMap, HashSet};

use crate::{
  asm::{DefinitionContent, FnLine, Function, Module, Register},
  instruction::Instruction,
};

pub fn remove_unused_registers(module: &mut Module) {
  for defn in &mut module.definitions {
    if let DefinitionContent::Function(fn_) = &mut defn.content {
      remove_unused_registers_fn(fn_);
    }
  }
}

fn remove_unused_registers_fn(fn_: &mut Function) {
  let dependency_tree = build_dependency_tree(fn_);
  let primary_registers = identify_primary_registers(fn_);
  let used_registers = gather_used_registers(&primary_registers, &dependency_tree);

  for line in &mut fn_.body {
    let instr = match line {
      FnLine::Instruction(instr) => instr,
      _ => continue,
    };

    let mut comment_out = false;

    instr.visit_registers_mut_rev(&mut |rvm| {
      if !used_registers.contains(&rvm.register.name) {
        if rvm.read {
          comment_out = true;
        } else if rvm.write {
          *rvm.register = Register::ignore();
        }
      }
    });

    if comment_out {
      *line = FnLine::Comment(line.to_string());
    }
  }
}

fn gather_used_registers(
  primary_registers: &HashSet<String>,
  dependency_tree: &HashMap<String, HashSet<String>>,
) -> HashSet<String> {
  let mut used_registers = HashSet::<String>::new();

  for reg in primary_registers {
    add_used_register(&mut used_registers, reg, dependency_tree);
  }

  used_registers
}

fn add_used_register(
  used_registers: &mut HashSet<String>,
  reg: &String,
  dependency_tree: &HashMap<String, HashSet<String>>,
) {
  if used_registers.contains(reg) {
    return;
  }

  used_registers.insert(reg.clone());

  if let Some(deps) = dependency_tree.get(reg) {
    for dep in deps {
      add_used_register(used_registers, dep, dependency_tree);
    }
  }
}

fn build_dependency_tree(fn_: &mut Function) -> HashMap<String, HashSet<String>> {
  let mut dependency_tree = HashMap::<String, HashSet<String>>::new();

  for line in &mut fn_.body {
    let instr = match line {
      FnLine::Instruction(instr) => instr,
      _ => continue,
    };

    let mut reads = HashSet::<String>::new();
    let mut writes = HashSet::<String>::new();

    instr.visit_registers_mut_rev(&mut |rvm| {
      if rvm.read {
        reads.insert(rvm.register.name.clone());
      }

      if rvm.write {
        writes.insert(rvm.register.name.clone());
      }
    });

    for write in &writes {
      let dependents = dependency_tree.entry(write.clone()).or_default();
      for read in &reads {
        dependents.insert(read.clone());
      }
    }
  }

  dependency_tree
}

fn identify_primary_registers(fn_: &mut Function) -> HashSet<String> {
  let mut primary_registers = HashSet::<String>::new();
  primary_registers.insert("return".to_string());
  primary_registers.insert("this".to_string());

  for line in &mut fn_.body {
    let instr = match line {
      FnLine::Instruction(instr) => instr,
      _ => continue,
    };

    match instr {
      Instruction::Call(fn_, args, _) => {
        fn_.visit_registers_mut_rev(&mut |rvm| {
          primary_registers.insert(rvm.register.name.clone());
        });

        args.visit_registers_mut_rev(&mut |rvm| {
          primary_registers.insert(rvm.register.name.clone());
        });
      }
      Instruction::Apply(fn_, ctx, args, _) => {
        fn_.visit_registers_mut_rev(&mut |rvm| {
          primary_registers.insert(rvm.register.name.clone());
        });

        primary_registers.insert(ctx.name.clone());

        args.visit_registers_mut_rev(&mut |rvm| {
          primary_registers.insert(rvm.register.name.clone());
        });
      }
      Instruction::SubCall(obj, key, args, _) | Instruction::ThisSubCall(obj, key, args, _) => {
        primary_registers.insert(obj.name.clone());

        key.visit_registers_mut_rev(&mut |rvm| {
          primary_registers.insert(rvm.register.name.clone());
        });

        args.visit_registers_mut_rev(&mut |rvm| {
          primary_registers.insert(rvm.register.name.clone());
        });
      }
      Instruction::ConstSubCall(obj, key, args, _) => {
        obj.visit_registers_mut_rev(&mut |rvm| {
          primary_registers.insert(rvm.register.name.clone());
        });

        key.visit_registers_mut_rev(&mut |rvm| {
          primary_registers.insert(rvm.register.name.clone());
        });

        args.visit_registers_mut_rev(&mut |rvm| {
          primary_registers.insert(rvm.register.name.clone());
        });
      }
      Instruction::Next(reg, _) => {
        primary_registers.insert(reg.name.clone());
      }
      Instruction::Cat(value, _)
      | Instruction::Yield(value, _)
      | Instruction::YieldStar(value, _)
      | Instruction::JmpIf(value, _)
      | Instruction::JmpIfNot(value, _)
      | Instruction::Throw(value) => {
        value.visit_registers_mut_rev(&mut |rvm| {
          primary_registers.insert(rvm.register.name.clone());
        });
      }

      _ => {}
    }
  }

  primary_registers
}
