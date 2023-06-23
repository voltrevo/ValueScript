use crate::asm::{Number, Value};

pub const CONSTANTS: [(&'static str, Value); 2] = [
  ("NaN", Value::Number(Number(std::f64::NAN))),
  ("Infinity", Value::Number(Number(std::f64::INFINITY))),
];
