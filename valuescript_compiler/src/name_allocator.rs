use std::collections::HashSet;

use crate::asm::{Pointer, Register};

#[derive(Default)]
pub struct NameAllocator {
  used_names: HashSet<String>,
  released_names: Vec<String>,
}

impl NameAllocator {
  pub fn allocate(&mut self, based_on_name: &String) -> String {
    match self.released_names.pop() {
      Some(name) => {
        // FIXME: When reallocating a register we need to ensure we don't read
        // the leftover value
        self.used_names.insert(name.clone());
        return name;
      }
      None => {}
    };

    return self.allocate_fresh(based_on_name);
  }

  pub fn allocate_fresh(&mut self, based_on_name: &String) -> String {
    if !self.used_names.contains(based_on_name) {
      self.used_names.insert(based_on_name.clone());
      return based_on_name.clone();
    }

    return self.allocate_numbered(&(based_on_name.clone() + "_"));
  }

  pub fn allocate_numbered(&mut self, prefix: &String) -> String {
    match self.released_names.pop() {
      Some(name) => {
        // FIXME: When reallocating a register we need to ensure we don't read
        // the leftover value
        self.used_names.insert(name.clone());
        return name;
      }
      None => {}
    };

    return self.allocate_numbered_fresh(prefix);
  }

  pub fn allocate_numbered_fresh(&mut self, prefix: &String) -> String {
    let mut i = 0_u64;

    loop {
      let candidate = prefix.clone() + &i.to_string();

      if !self.used_names.contains(&candidate) {
        self.used_names.insert(candidate.clone());
        return candidate;
      }

      i += 1;
    }
  }

  pub fn release(&mut self, name: &String) {
    self.used_names.remove(name);
    self.released_names.push(name.clone());
  }
}

#[derive(Default)]
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

#[derive(Default)]
pub struct RegAllocator {
  alloc: NameAllocator,
}

impl RegAllocator {
  pub fn allocate(&mut self, based_on_name: &str) -> Register {
    let name = self.alloc.allocate(&based_on_name.to_string());
    Register::Named(name)
  }
}
