use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use super::function_compiler::QueuedFunction;

#[derive(Clone, Debug)]
pub enum Builtin {
  Math,
}

impl std::fmt::Display for Builtin {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

#[derive(Clone, Debug)]
pub enum MappedName {
  Register(String),
  Definition(String),
  QueuedFunction(QueuedFunction),
  Builtin(Builtin),
}

pub struct ScopeData {
  pub name_map: HashMap<String, MappedName>,
  pub parent: Option<Rc<RefCell<ScopeData>>>,
}

pub type Scope = Rc<RefCell<ScopeData>>;

pub trait ScopeTrait {
  fn get(&self, name: &String) -> Option<MappedName>;
  fn set(&self, name: String, mapped_name: MappedName);
  fn nest(&self) -> Rc<RefCell<ScopeData>>;
}

impl ScopeTrait for Scope {
  fn get(&self, name: &String) -> Option<MappedName> {
    match self.borrow().name_map.get(name) {
      Some(mapped_name) => Some(mapped_name.clone()),
      None => match &self.borrow().parent {
        Some(parent) => parent.get(name),
        None => None,
      },
    }
  }

  fn set(&self, name: String, mapped_name: MappedName) {
    let old_mapping = self.borrow_mut().name_map.insert(name, mapped_name);

    if old_mapping.is_some() {
      std::panic!("Scope overwrite occurred (not implemented: being permissive about this)");
    }
  }

  fn nest(&self) -> Rc<RefCell<ScopeData>> {
    return Rc::new(RefCell::new(ScopeData {
      name_map: Default::default(),
      parent: Some(self.clone()),
    }));
  }
}

pub fn init_scope() -> Scope {
  return Rc::new(RefCell::new(ScopeData {
    name_map: Default::default(),
    parent: None,
  }));
}

pub fn init_std_scope() -> Scope {
  return Rc::new(RefCell::new(ScopeData {
    name_map: HashMap::from([
      ("Math".to_string(), MappedName::Builtin(Builtin::Math)),
    ]),
    parent: None,
  }));
}
