use std::collections::HashSet;

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
      },
      None => {},
    };

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
      },
      None => {},
    };

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
