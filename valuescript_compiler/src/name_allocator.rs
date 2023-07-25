use std::collections::BTreeSet;

use crate::asm::{Pointer, Register};

#[derive(Default, Clone, Debug)]
pub struct NameAllocator {
  used_names: BTreeSet<String>,
  released_names: Vec<String>,
}

impl NameAllocator {
  pub fn allocate(&mut self, based_on_name: &String) -> String {
    if let Some(name) = self.released_names.pop() {
      // FIXME: When reallocating a register we need to ensure we don't read
      // the leftover value
      self.used_names.insert(name.clone());
      return name;
    };

    self.allocate_fresh(based_on_name)
  }

  pub fn allocate_fresh(&mut self, based_on_name: &String) -> String {
    if !self.used_names.contains(based_on_name) {
      self.used_names.insert(based_on_name.clone());
      return based_on_name.clone();
    }

    self.allocate_numbered(&(based_on_name.clone() + "_"))
  }

  pub fn allocate_numbered(&mut self, prefix: &str) -> String {
    if let Some(name) = self.released_names.pop() {
      // FIXME: When reallocating a register we need to ensure we don't read
      // the leftover value
      self.used_names.insert(name.clone());
      return name;
    };

    self.allocate_numbered_fresh(prefix)
  }

  pub fn allocate_numbered_fresh(&mut self, prefix: &str) -> String {
    let mut i = 0_u64;

    loop {
      let candidate = prefix.to_string() + &i.to_string();

      if !self.used_names.contains(&candidate) {
        self.used_names.insert(candidate.clone());
        return candidate;
      }

      i += 1;
    }
  }

  pub fn mark_used(&mut self, name: &str) {
    self.used_names.insert(name.to_string());
  }

  pub fn release(&mut self, name: &String) {
    self.used_names.remove(name);
    self.released_names.push(name.clone());
  }
}

pub fn ident_from_str(str: &str) -> String {
  let mut res = "".to_string();
  let mut first = false;
  let mut last_sep = false;

  for c in str.chars() {
    if first {
      first = false;

      if c.is_ascii_alphabetic() {
        res.push(c);
        continue;
      }

      res.push('_');
      last_sep = true;
    }

    if !c.is_ascii_alphanumeric() {
      if !last_sep {
        res.push('_');
        last_sep = true;
      }

      continue;
    }

    res.push(c);
    last_sep = false;
  }

  match last_sep && res.len() > 1 {
    false => res,
    true => res[0..res.len() - 1].to_string(),
  }
}

#[derive(Default, Debug)]
pub struct PointerAllocator {
  alloc: NameAllocator,
}

impl PointerAllocator {
  pub fn allocate(&mut self, based_on_name: &str) -> Pointer {
    Pointer {
      name: self.alloc.allocate(&based_on_name.to_string()),
    }
  }
}

#[derive(Clone, Debug)]
pub struct RegAllocator {
  pub alloc: NameAllocator,
}

impl RegAllocator {
  pub fn allocate(&mut self, based_on_name: &str) -> Register {
    let name = self.alloc.allocate(&based_on_name.to_string());
    Register::named(name)
  }

  pub fn allocate_fresh(&mut self, based_on_name: &str) -> Register {
    let name = self.alloc.allocate_fresh(&based_on_name.to_string());
    Register::named(name)
  }

  pub fn allocate_numbered(&mut self, prefix: &str) -> Register {
    let name = self.alloc.allocate_numbered(prefix);
    Register::named(name)
  }

  pub fn allocate_numbered_fresh(&mut self, prefix: &str) -> Register {
    let name = self.alloc.allocate_numbered_fresh(prefix);
    Register::named(name)
  }

  // TODO: We used to release names back into the allocator a lot. However, the optimizer now
  // extends lifetimes, which means re-purposing a register is no longer safe. The optimizer needs
  // to do its own analysis to restore register re-use, because we're now using way too many.
  // Anyway, marking this as dead code for now but maybe it should just be removed.
  #[allow(dead_code)]
  pub fn release(&mut self, reg: &Register) {
    match reg.is_named() {
      true => self.alloc.release(&reg.name),
      false => panic!("Can't release non-named register"),
    }
  }

  pub fn all_used(&self) -> Vec<Register> {
    let mut res = Vec::<Register>::new();

    for name in &self.alloc.used_names {
      res.push(Register::named(name.clone()));
    }

    res
  }
}

impl Default for RegAllocator {
  fn default() -> Self {
    let mut alloc = NameAllocator::default();
    alloc.allocate(&"return".to_string());
    alloc.allocate(&"this".to_string());
    alloc.allocate(&"ignore".to_string());

    RegAllocator { alloc }
  }
}
