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

  let single_return_dep = {
    match dependency_tree.get("return") {
      Some(deps) => match deps.iter().next() {
        Some(dep) => match deps.len() {
          1 => Some(dep.clone()),
          _ => None,
        },
        None => None,
      },
      None => None,
    }
  };

  'b: {
    if let Some(single_return_dep) = single_return_dep {
      if &single_return_dep == "this" {
        break 'b;
      }

      let return_string = "return".to_string();

      for deps in dependency_tree.values() {
        if deps.contains(&return_string) {
          break 'b;
        }
      }

      for param in &fn_.parameters {
        if &param.name == &single_return_dep {
          break 'b;
        }
      }

      // FIXME: This logic is not ideal. I'm not sure it's 100% correct. The idea is to fix the case
      // where `single_return_dep` is used after returning it. You could certainly break it with
      // handwritten assembly.
      //
      // In future, this return renaming should be replaced by a more robust logic from control flow
      // analysis, which can determine that the returned value always coincides with one other
      // register.
      //
      if !return_dep_always_taken(&mut fn_.body, &single_return_dep) {
        break 'b;
      }

      rename_register_in_body(&mut fn_.body, &single_return_dep, &return_string);
    }
  }
}

fn return_dep_always_taken(body: &mut Vec<FnLine>, dep: &String) -> bool {
  let return_string = "return".to_string();

  for line in body {
    let instr = match line {
      FnLine::Instruction(instr) => instr,
      _ => continue,
    };

    let mut reads_dep_copy = false;
    let mut writes_return = false;

    instr.visit_registers_mut_rev(&mut |rvm| {
      if rvm.read && &rvm.register.name == dep && !rvm.register.take {
        reads_dep_copy = true;
      }

      if rvm.write && rvm.register.name == return_string {
        writes_return = true;
      }
    });

    if reads_dep_copy && writes_return {
      return false;
    }
  }

  return true;
}

fn rename_register_in_body(body: &mut Vec<FnLine>, from: &String, to: &String) {
  // Note: This only does the rename in the body and basically assumes that %from is not a
  // parameter.

  for line in body {
    let instr = match line {
      FnLine::Instruction(instr) => instr,
      _ => continue,
    };

    instr.visit_registers_mut_rev(&mut |rvm| {
      if &rvm.register.name == from {
        // TODO: Preserving `.take=true` can cause problems. Is just using `.take=false` the right
        // solution?
        *rvm.register = Register::named(to.clone());
      }
    });
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
