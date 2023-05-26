use std::rc::Rc;

use num_bigint::BigInt;

use crate::{
  vs_array::VsArray,
  vs_class::VsClass,
  vs_object::VsObject,
  vs_symbol::VsSymbol,
  vs_value::{LoadFunctionResult, ToValString, Val, VsType},
  ValTrait,
};

use super::type_error_builtin::ToTypeError;

pub struct SymbolBuiltin {}

pub static SYMBOL_BUILTIN: SymbolBuiltin = SymbolBuiltin {};

impl ValTrait for SymbolBuiltin {
  fn typeof_(&self) -> VsType {
    VsType::Object
  }
  fn val_to_string(&self) -> String {
    "[object Symbol]".to_string()
  }
  fn to_number(&self) -> f64 {
    core::f64::NAN
  }
  fn to_index(&self) -> Option<usize> {
    None
  }
  fn is_primitive(&self) -> bool {
    false
  }
  fn to_primitive(&self) -> Val {
    self.to_val_string()
  }
  fn is_truthy(&self) -> bool {
    true
  }
  fn is_nullish(&self) -> bool {
    false
  }
  fn bind(&self, _params: Vec<Val>) -> Option<Val> {
    None
  }
  fn as_bigint_data(&self) -> Option<BigInt> {
    None
  }
  fn as_array_data(&self) -> Option<Rc<VsArray>> {
    None
  }
  fn as_object_data(&self) -> Option<Rc<VsObject>> {
    None
  }
  fn as_class_data(&self) -> Option<Rc<VsClass>> {
    None
  }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn sub(&self, key: Val) -> Result<Val, Val> {
    Ok(match key.val_to_string().as_str() {
      "iterator" => Val::Symbol(VsSymbol::ITERATOR),
      _ => Val::Undefined,
    })
  }

  fn submov(&mut self, _key: Val, _value: Val) -> Result<(), Val> {
    Err("Cannot assign to subscript of Symbol builtin".to_type_error())
  }

  fn next(&mut self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Symbol]\x1b[39m")
  }

  fn codify(&self) -> String {
    "Symbol".into()
  }
}
