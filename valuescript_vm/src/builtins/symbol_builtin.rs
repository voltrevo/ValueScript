use std::{fmt, rc::Rc};

use crate::{
  vs_class::VsClass,
  vs_symbol::VsSymbol,
  vs_value::{LoadFunctionResult, ToVal, Val},
};

use super::builtin_object::BuiltinObject;

pub struct SymbolBuiltin {}

impl BuiltinObject for SymbolBuiltin {
  fn bo_name() -> &'static str {
    "Symbol"
  }

  fn bo_sub(key: &str) -> Val {
    match key {
      "iterator" => VsSymbol::ITERATOR.to_val(),
      _ => Val::Undefined,
    }
  }

  fn bo_load_function() -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn bo_as_class_data() -> Option<Rc<VsClass>> {
    None
  }
}

impl fmt::Display for SymbolBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[object Symbol]")
  }
}
