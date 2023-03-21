use std::rc::Rc;
use std::str::FromStr;

use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;
use num_traits::Zero;

use super::operations::{op_sub, op_submov};
use super::stack_frame::StackFrame;
use super::vs_array::VsArray;
use super::vs_class::VsClass;
use super::vs_function::VsFunction;
use super::vs_object::VsObject;

#[derive(Clone)]
pub enum Val {
  Void,
  Undefined,
  Null,
  Bool(bool),
  Number(f64),
  BigInt(BigInt),
  String(Rc<String>),
  Array(Rc<VsArray>),
  Object(Rc<VsObject>),
  Function(Rc<VsFunction>),
  Class(Rc<VsClass>),
  Static(&'static dyn ValTrait),
  Custom(Rc<dyn ValTrait>),
}

#[derive(PartialEq, Debug)]
pub enum VsType {
  Undefined,
  Null,
  Bool,
  Number,
  BigInt,
  String,
  Array,
  Object,
  Function,
  Class,
}

pub enum LoadFunctionResult {
  NotAFunction,
  StackFrame(StackFrame),
  NativeFunction(fn(this: &mut Val, params: Vec<Val>) -> Result<Val, Val>),
}

pub trait ValTrait {
  fn typeof_(&self) -> VsType;
  fn val_to_string(&self) -> String;
  fn to_number(&self) -> f64;
  fn to_index(&self) -> Option<usize>;
  fn is_primitive(&self) -> bool;
  fn to_primitive(&self) -> Val;
  fn is_truthy(&self) -> bool;
  fn is_nullish(&self) -> bool;

  fn bind(&self, params: Vec<Val>) -> Option<Val>;

  fn as_bigint_data(&self) -> Option<BigInt>;
  fn as_array_data(&self) -> Option<Rc<VsArray>>;
  fn as_object_data(&self) -> Option<Rc<VsObject>>;
  fn as_class_data(&self) -> Option<Rc<VsClass>>;

  fn load_function(&self) -> LoadFunctionResult;

  fn sub(&self, key: Val) -> Result<Val, Val>;
  fn submov(&mut self, key: Val, value: Val) -> Result<(), Val>;

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
  fn codify(&self) -> String;
}

impl ValTrait for Val {
  fn typeof_(&self) -> VsType {
    use Val::*;

    return match self {
      Void => VsType::Undefined,
      Undefined => VsType::Undefined,
      Null => VsType::Null,
      Bool(_) => VsType::Bool,
      Number(_) => VsType::Number,
      BigInt(_) => VsType::BigInt,
      String(_) => VsType::String,
      Array(_) => VsType::Array,
      Object(_) => VsType::Object,
      Function(_) => VsType::Function,
      Class(_) => VsType::Class,
      Static(val) => val.typeof_(),
      Custom(val) => val.typeof_(),
    };
  }

  fn val_to_string(&self) -> String {
    use Val::*;

    return match self {
      Void => "".to_string(),
      Undefined => "undefined".to_string(),
      Null => "null".to_string(),
      Bool(b) => b.to_string(),
      Number(x) => {
        if x.is_infinite() {
          if x.is_sign_positive() {
            "Infinity".to_string()
          } else {
            "-Infinity".to_string()
          }
        } else {
          x.to_string()
        }
      } // TODO: Match js's number string format
      BigInt(x) => x.to_string(),
      String(s) => s.to_string(),
      Array(vals) => {
        if vals.elements.len() == 0 {
          "".to_string()
        } else if vals.elements.len() == 1 {
          vals.elements[0].val_to_string()
        } else {
          let mut iter = vals.elements.iter();
          let mut res = iter.next().unwrap().val_to_string();

          for val in iter {
            res += ",";

            match val.typeof_() {
              VsType::Undefined => {}
              _ => {
                res += &val.val_to_string();
              }
            };
          }

          res
        }
      }
      Object(_) => "[object Object]".to_string(),
      Function(_) => "[function]".to_string(),
      Class(_) => "[class]".to_string(),
      Static(val) => val.val_to_string(),
      Custom(val) => val.val_to_string(),
    };
  }

  fn to_number(&self) -> f64 {
    use Val::*;

    return match self {
      Void => f64::NAN,
      Undefined => f64::NAN,
      Null => 0_f64,
      Bool(b) => *b as u8 as f64,
      Number(x) => *x,
      BigInt(x) => x.to_f64().unwrap_or(f64::NAN),
      String(s) => f64::from_str(s).unwrap_or(f64::NAN),
      Array(vals) => match vals.elements.len() {
        0 => 0_f64,
        1 => vals.elements[0].to_number(),
        _ => f64::NAN,
      },
      Object(_) => f64::NAN,
      Function(_) => f64::NAN,
      Class(_) => f64::NAN,
      Static(val) => val.to_number(),
      Custom(val) => val.to_number(),
    };
  }

  fn to_index(&self) -> Option<usize> {
    use Val::*;

    return match self {
      Void => panic!("Shouldn't happen"),
      Undefined => None,
      Null => None,
      Bool(_) => None,
      Number(x) => number_to_index(*x),
      BigInt(b) => number_to_index(b.to_f64().unwrap_or(f64::NAN)),
      String(s) => match f64::from_str(s) {
        Ok(x) => number_to_index(x),
        Err(_) => None,
      },
      Array(_) => None,
      Object(_) => None,
      Function(_) => None,
      Class(_) => None,
      Static(val) => val.to_index(),
      Custom(val) => val.to_index(),
    };
  }

  fn is_primitive(&self) -> bool {
    use Val::*;

    return match self {
      Void => true,
      Undefined => true,
      Null => true,
      Bool(_) => true,
      Number(_) => true,
      BigInt(_) => true,
      String(_) => true,
      Array(_) => false,
      Object(_) => false,
      Function(_) => false,
      Class(_) => false,
      Static(val) => val.is_primitive(), // TODO: false?
      Custom(val) => val.is_primitive(),
    };
  }

  fn to_primitive(&self) -> Val {
    if self.is_primitive() {
      return self.clone();
    }

    return Val::String(Rc::new(self.val_to_string()));
  }

  fn is_truthy(&self) -> bool {
    use Val::*;

    return match self {
      Void => false,
      Undefined => false,
      Null => false,
      Bool(b) => *b,
      Number(x) => *x != 0_f64 && !x.is_nan(),
      BigInt(x) => !x.is_zero(),
      String(s) => s.len() > 0,
      Array(_) => true,
      Object(_) => true,
      Function(_) => true,
      Class(_) => true,
      Static(val) => val.is_truthy(), // TODO: true?
      Custom(val) => val.is_truthy(),
    };
  }

  fn is_nullish(&self) -> bool {
    use Val::*;

    return match self {
      Void => panic!("Shouldn't happen"), // TODO: Or just true?
      Undefined => true,
      Null => true,
      Bool(_) => false,
      Number(_) => false,
      BigInt(_) => false,
      String(_) => false,
      Array(_) => false,
      Object(_) => false,
      Function(_) => false,
      Class(_) => false,
      Static(_) => false,
      Custom(val) => val.is_nullish(),
    };
  }

  fn bind(&self, params: Vec<Val>) -> Option<Val> {
    use Val::*;

    return match self {
      Function(f) => Some(Val::Function(Rc::new(f.bind(params)))),
      Custom(val) => val.bind(params),

      _ => None,
    };
  }

  fn as_bigint_data(&self) -> Option<BigInt> {
    use Val::*;

    return match self {
      BigInt(b) => Some(b.clone()),
      // TODO: Static? Others too?
      Custom(val) => val.as_bigint_data(),

      _ => None,
    };
  }

  fn as_array_data(&self) -> Option<Rc<VsArray>> {
    use Val::*;

    return match self {
      Array(a) => Some(a.clone()),
      Custom(val) => val.as_array_data(),

      _ => None,
    };
  }

  fn as_object_data(&self) -> Option<Rc<VsObject>> {
    use Val::*;

    return match self {
      Object(obj) => Some(obj.clone()),
      Custom(val) => val.as_object_data(),

      _ => None,
    };
  }

  fn as_class_data(&self) -> Option<Rc<VsClass>> {
    use Val::*;

    return match self {
      Class(class) => Some(class.clone()),
      Custom(val) => val.as_class_data(),

      _ => None,
    };
  }

  fn load_function(&self) -> LoadFunctionResult {
    use Val::*;

    return match self {
      Function(f) => LoadFunctionResult::StackFrame(f.make_frame()),
      Static(s) => s.load_function(),
      Custom(val) => val.load_function(),

      _ => LoadFunctionResult::NotAFunction,
    };
  }

  fn sub(&self, key: Val) -> Result<Val, Val> {
    // TODO: Avoid cloning?
    op_sub(self.clone(), key)
  }

  fn submov(&mut self, key: Val, value: Val) -> Result<(), Val> {
    op_submov(self, key, value)
  }

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(self, f)
  }

  fn codify(&self) -> String {
    match self {
      Val::Void => "".into(),
      Val::Undefined => "undefined".into(),
      Val::Null => "null".into(),
      Val::Bool(_) => self.val_to_string(),
      Val::Number(_) => self.val_to_string(),
      Val::BigInt(_) => self.val_to_string() + "n",
      Val::String(str) => stringify_string(str),
      Val::Array(vals) => {
        if vals.elements.len() == 0 {
          "[]".to_string()
        } else if vals.elements.len() == 1 {
          "[".to_string() + vals.elements[0].codify().as_str() + "]"
        } else {
          let mut iter = vals.elements.iter();
          let mut res: String = "[".into();
          res += iter.next().unwrap().codify().as_str();

          for val in iter {
            res += ",";
            res += &val.codify();
          }

          res += "]";

          res
        }
      }
      Val::Object(object) => {
        if object.string_map.len() == 0 {
          return "{}".into();
        }

        let mut res = "{".into();
        let mut first = true;

        for (k, v) in &object.string_map {
          if first {
            first = false;
          } else {
            res += ",";
          }

          res += stringify_string(k).as_str();
          res += ":";
          res += v.codify().as_str();
        }

        res += "}";

        res
      }
      Val::Function(_) => "() => { [unavailable] }".to_string(),
      Val::Class(_) => "class { [unavailable] }".to_string(),
      Val::Static(val) => val.codify(),
      Val::Custom(val) => val.codify(),
    }
  }
}

impl std::fmt::Display for Val {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Val::Void => write!(f, "void"),
      Val::Undefined => write!(f, "\x1b[90mundefined\x1b[39m"),
      Val::Null => write!(f, "\x1b[1mnull\x1b[22m"),
      Val::Bool(_) => write!(f, "\x1b[33m{}\x1b[39m", self.val_to_string()),
      Val::Number(_) => write!(f, "\x1b[33m{}\x1b[39m", self.val_to_string()),
      Val::BigInt(_) => write!(f, "\x1b[33m{}n\x1b[39m", self.val_to_string()),
      Val::String(_) => write!(f, "\x1b[32m{}\x1b[39m", self.codify()),
      Val::Array(array) => {
        if array.elements.len() == 0 {
          return write!(f, "[]");
        }

        write!(f, "[ ").expect("Failed to write");

        let mut first = true;

        for elem in &array.elements {
          if first {
            first = false;
          } else {
            write!(f, ", ").expect("Failed to write");
          }

          write!(f, "{}", elem).expect("Failed to write");
        }

        write!(f, " ]")
      }
      Val::Object(object) => {
        if object.string_map.len() == 0 {
          return f.write_str("{}");
        }

        match f.write_str("{ ") {
          Ok(_) => {}
          Err(e) => {
            return Err(e);
          }
        };

        let mut first = true;

        for (k, v) in &object.string_map {
          if first {
            first = false;
          } else {
            write!(f, ", ").expect("Failed to write");
          }

          write!(f, "{}: {}", k, v).expect("Failed to write");
        }

        f.write_str(" }")
      }
      Val::Function(_) => write!(f, "\x1b[36m[Function]\x1b[39m"),
      Val::Class(_) => write!(f, "\x1b[36m[Class]\x1b[39m"),

      // TODO: Improve printing these
      Val::Static(s) => s.fmt(f),
      Val::Custom(c) => c.fmt(f),
    }
  }
}

fn number_to_index(x: f64) -> Option<usize> {
  if x < 0_f64 || x != x.floor() {
    return None;
  }

  return Some(x as usize);
}

fn stringify_string(str: &String) -> String {
  let mut res: String = "\"".into();

  for c in str.chars() {
    let escape_seq = match c {
      '\r' => Some("\\r"),
      '\n' => Some("\\n"),
      '\t' => Some("\\t"),
      '"' => Some("\\\""),
      _ => None,
    };

    match escape_seq {
      Some(seq) => {
        res += seq;
      }
      None => {
        res.push(c);
      }
    };
  }

  res += "\"";

  res
}
