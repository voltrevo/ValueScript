mod array_builtin;
mod boolean_builtin;
mod builtin_object;
mod debug_builtin;
pub mod error_builtin;
mod math_builtin;
mod number_builtin;
pub mod range_error_builtin;
mod string_builtin;
mod symbol_builtin;
pub mod type_error_builtin;

use valuescript_common::BUILTIN_COUNT;

use crate::{
  vs_symbol::VsSymbol,
  vs_value::{ToVal, Val},
};

use self::{
  array_builtin::ArrayBuiltin, boolean_builtin::BooleanBuiltin, debug_builtin::DebugBuiltin,
  error_builtin::ErrorBuiltin, math_builtin::MathBuiltin, number_builtin::NumberBuiltin,
  range_error_builtin::RangeErrorBuiltin, string_builtin::StringBuiltin,
  symbol_builtin::SymbolBuiltin, type_error_builtin::TypeErrorBuiltin,
};

pub static BUILTIN_VALS: [fn() -> Val; BUILTIN_COUNT] = [
  || DebugBuiltin {}.to_val(),
  || MathBuiltin {}.to_val(),
  || StringBuiltin {}.to_val(),
  || NumberBuiltin {}.to_val(),
  || BooleanBuiltin {}.to_val(),
  || ArrayBuiltin {}.to_val(),
  || number_builtin::IS_FINITE.to_val(),
  || number_builtin::IS_NAN.to_val(),
  || number_builtin::PARSE_FLOAT.to_val(),
  || number_builtin::PARSE_INT.to_val(),
  || ErrorBuiltin {}.to_val(),
  || TypeErrorBuiltin {}.to_val(),
  || RangeErrorBuiltin {}.to_val(),
  || SymbolBuiltin {}.to_val(),
  || VsSymbol::ITERATOR.to_val(),
];
