use crate::asm::Value;

pub const CONSTANTS: [(&'static str, Value); 2] = [
  ("NaN", Value::Number(std::f64::NAN)),
  ("Infinity", Value::Number(std::f64::INFINITY)),
];
