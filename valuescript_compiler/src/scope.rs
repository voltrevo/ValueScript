use std::{cell::RefCell, collections::HashMap, rc::Rc};

use swc_common::Spanned;

use valuescript_common::BUILTIN_NAMES;

use crate::diagnostic::{Diagnostic, DiagnosticLevel};
use crate::{asm::Builtin, constants::CONSTANTS};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum NameId {
  Span(swc_common::Span),
  Builtin(Builtin),
  Constant(&'static str),
}

impl Spanned for NameId {
  fn span(&self) -> swc_common::Span {
    match self {
      NameId::Span(span) => *span,
      NameId::Builtin(_) => swc_common::DUMMY_SP,
      NameId::Constant(_) => swc_common::DUMMY_SP,
    }
  }
}

pub struct ScopeData {
  pub owner_id: OwnerId,
  pub name_map: HashMap<swc_atoms::JsWord, NameId>,
  pub parent: Option<Rc<RefCell<ScopeData>>>,
}

pub type Scope = Rc<RefCell<ScopeData>>;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum OwnerId {
  Span(swc_common::Span),
  Module,
}

pub trait ScopeTrait {
  fn get(&self, name: &swc_atoms::JsWord) -> Option<NameId>;
  fn set(
    &self,
    name: &swc_atoms::JsWord,
    name_id: NameId,
    span: swc_common::Span,
    diagnostics: &mut Vec<Diagnostic>,
  );
  fn nest(&self, name_owner_location: Option<OwnerId>) -> Rc<RefCell<ScopeData>>;
}

impl ScopeTrait for Scope {
  fn get(&self, name: &swc_atoms::JsWord) -> Option<NameId> {
    match self.borrow().name_map.get(name) {
      Some(mapped_name) => Some(mapped_name.clone()),
      None => match &self.borrow().parent {
        Some(parent) => parent.get(name),
        None => None,
      },
    }
  }

  fn set(
    &self,
    name: &swc_atoms::JsWord,
    name_id: NameId,
    span: swc_common::Span,
    diagnostics: &mut Vec<Diagnostic>,
  ) {
    let old_mapping = self.borrow_mut().name_map.insert(name.clone(), name_id);

    if old_mapping.is_some() {
      diagnostics.push(Diagnostic {
        level: DiagnosticLevel::Error,
        message: "Scope overwrite occurred (TODO: being permissive about this)".to_string(),
        span,
      });
    }
  }

  fn nest(&self, name_owner_location: Option<OwnerId>) -> Rc<RefCell<ScopeData>> {
    return Rc::new(RefCell::new(ScopeData {
      owner_id: name_owner_location.unwrap_or(self.borrow().owner_id.clone()),
      name_map: Default::default(),
      parent: Some(self.clone()),
    }));
  }
}

pub fn init_std_scope() -> Scope {
  let mut name_map = HashMap::new();

  for name in BUILTIN_NAMES {
    name_map.insert(
      swc_atoms::JsWord::from(name),
      NameId::Builtin(Builtin {
        name: name.to_string(),
      }),
    );
  }

  for (name, _) in CONSTANTS {
    name_map.insert(swc_atoms::JsWord::from(name), NameId::Constant(name));
  }

  Rc::new(RefCell::new(ScopeData {
    owner_id: OwnerId::Module,
    name_map,
    parent: None,
  }))
  .nest(None)
}
