use crate::asm::{Number, Value};

pub const CONSTANTS: [(&str, Value); 3] = [
  ("undefined", Value::Undefined),
  ("NaN", Value::Number(Number(std::f64::NAN))),
  ("Infinity", Value::Number(Number(std::f64::INFINITY))),
];
