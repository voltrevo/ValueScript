use std::{fmt, rc::Rc};

use crate::{
  vs_class::VsClass,
  vs_symbol::VsSymbol,
  vs_value::{LoadFunctionResult, Val},
};

use super::builtin_object::BuiltinObject;

pub struct SymbolBuiltin {}

pub static SYMBOL_BUILTIN: SymbolBuiltin = SymbolBuiltin {};

impl BuiltinObject for SymbolBuiltin {
  fn bo_name() -> &'static str {
    "Symbol"
  }

  fn bo_sub(key: &str) -> Val {
    match key {
      "iterator" => Val::Symbol(VsSymbol::ITERATOR),
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
