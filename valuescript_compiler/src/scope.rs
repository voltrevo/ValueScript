use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use valuescript_common::BUILTIN_NAMES;

use crate::{
  asm::{Builtin, Pointer, Register, Value},
  constants::CONSTANTS,
};

use super::function_compiler::QueuedFunction;

#[derive(Clone, Debug)]
pub enum MappedName {
  Register(Register),
  Definition(Pointer),
  QueuedFunction(QueuedFunction),
  Builtin(Builtin),
  Constant(Value),
}

pub fn scope_reg(name: String) -> MappedName {
  if name == "return" || name == "this" || name == "ignore" {
    std::panic!("Invalid register name (use Register enum)");
  }

  MappedName::Register(Register::Named(name))
}

pub struct ScopeData {
  pub name_map: HashMap<String, MappedName>,
  pub parent: Option<Scope>,
}

#[derive(Clone)]
pub struct Scope {
  pub rc: Rc<RefCell<ScopeData>>,
}

impl Scope {
  pub fn get(&self, name: &String) -> Option<MappedName> {
    match self.rc.borrow().name_map.get(name) {
      Some(mapped_name) => Some(mapped_name.clone()),
      None => match &self.rc.borrow().parent {
        Some(parent) => parent.get(name),
        None => None,
      },
    }
  }

  pub fn get_defn(&self, name: &String) -> Option<Pointer> {
    let get_result = self.get(name);

    return match get_result {
      Some(MappedName::Definition(d)) => Some(d.clone()),
      _ => None,
    };
  }

  pub fn set(&self, name: String, mapped_name: MappedName) {
    let old_mapping = self.rc.borrow_mut().name_map.insert(name, mapped_name);

    if old_mapping.is_some() {
      std::panic!("Scope overwrite occurred (not implemented: being permissive about this)");
    }
  }

  pub fn nest(&self) -> Scope {
    return Scope {
      rc: Rc::new(RefCell::new(ScopeData {
        name_map: Default::default(),
        parent: Some(self.clone()),
      })),
    };
  }
}

pub fn _init_scope() -> Scope {
  return Scope {
    rc: Rc::new(RefCell::new(ScopeData {
      name_map: Default::default(),
      parent: None,
    })),
  };
}

pub fn init_std_scope() -> Scope {
  let mut name_map: HashMap<String, MappedName> = Default::default();

  for name in BUILTIN_NAMES {
    name_map.insert(
      name.to_string(),
      MappedName::Builtin(Builtin {
        name: name.to_string(),
      }),
    );
  }

  for (name, value) in CONSTANTS {
    name_map.insert(name.to_string(), MappedName::Constant(value.clone()));
  }

  Scope {
    rc: Rc::new(RefCell::new(ScopeData {
      name_map,
      parent: None,
    })),
  }
  .nest()
}
