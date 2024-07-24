use crate::asm::{Number, Value};

pub const CONSTANTS: [(&str, Value); 3] = [
  ("undefined", Value::Undefined),
  ("NaN", Value::Number(Number(f64::NAN))),
  ("Infinity", Value::Number(Number(f64::INFINITY))),
];
