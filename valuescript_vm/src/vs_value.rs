use core::fmt;
use std::rc::Rc;
use std::str::FromStr;

use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;
use num_traits::Zero;

use crate::native_function::ThisWrapper;
use crate::operations::{op_sub, op_submov};
use crate::stack_frame::StackFrame;
use crate::vs_array::VsArray;
use crate::vs_class::VsClass;
use crate::vs_function::VsFunction;
use crate::vs_object::VsObject;
use crate::vs_symbol::{symbol_to_name, VsSymbol};

#[derive(Clone, Debug)]
pub enum Val {
  Void,
  Undefined,
  Null,
  Bool(bool),
  Number(f64),
  BigInt(BigInt),
  Symbol(VsSymbol),
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
  Symbol,
  String,
  Array,
  Object,
  Function,
  Class,
}

pub enum LoadFunctionResult {
  NotAFunction,
  StackFrame(StackFrame),
  NativeFunction(fn(this: ThisWrapper, params: Vec<Val>) -> Result<Val, Val>),
}

pub trait ValTrait: fmt::Display {
  fn typeof_(&self) -> VsType;
  fn to_number(&self) -> f64;
  fn to_index(&self) -> Option<usize>;
  fn is_primitive(&self) -> bool;
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

  fn next(&mut self) -> LoadFunctionResult;

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
  fn codify(&self) -> String;
}

impl fmt::Debug for dyn ValTrait {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "(dyn ValTrait)(")?;
    self.pretty_fmt(f)?;
    write!(f, ")")
  }
}

impl Val {
  pub fn to_primitive(&self) -> Val {
    if self.is_primitive() {
      self.clone()
    } else {
      self.clone().to_val_string()
    }
  }

  pub fn to_val_string(self) -> Val {
    match self {
      Val::String(_) => self,
      _ => self.to_string().to_val(),
    }
  }
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
      Symbol(_) => VsType::Symbol,
      String(_) => VsType::String,
      Array(_) => VsType::Array,
      Object(_) => VsType::Object,
      Function(_) => VsType::Function,
      Class(_) => VsType::Class,
      Static(val) => val.typeof_(),
      Custom(val) => val.typeof_(),
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
      Symbol(_) => f64::NAN, // TODO: Should be TypeError
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
      Symbol(_) => None,
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
      Symbol(_) => true,
      String(_) => true,
      Array(_) => false,
      Object(_) => false,
      Function(_) => false,
      Class(_) => false,
      Static(val) => val.is_primitive(), // TODO: false?
      Custom(val) => val.is_primitive(),
    };
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
      Symbol(_) => true,
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
      Symbol(_) => false,
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
      Function(f) => Some(f.bind(params).to_val()),
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
      Static(s) => s.as_class_data(),
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

  fn next(&mut self) -> LoadFunctionResult {
    match self {
      // TODO: iterator
      _ => {
        let next_fn = op_sub(self.clone(), "next".to_val());

        match next_fn {
          Ok(next_fn) => next_fn.load_function(),
          Err(_) => LoadFunctionResult::NotAFunction,
        }
      }
    }
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(&self.pretty(), f)
  }

  fn codify(&self) -> String {
    match self {
      Val::Void => "".into(),
      Val::Undefined => "undefined".into(),
      Val::Null => "null".into(),
      Val::Bool(_) => self.to_string(),
      Val::Number(_) => self.to_string(),
      Val::BigInt(_) => self.to_string() + "n",
      Val::Symbol(s) => format!("Symbol.{}", symbol_to_name(s.clone())),
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
        let mut res = String::new();

        if let Some(proto) = &object.prototype {
          match op_sub(proto.clone(), "name".to_val()) {
            Ok(name) => {
              if name.typeof_() == VsType::String {
                res += &name.to_string();
              }
            }
            Err(_) => {}
          }
        }

        if object.string_map.len() == 0 {
          res += "{}";
          return res;
        }

        res += "{";

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

impl fmt::Display for Val {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Val::*;

    match self {
      Void => Ok(()),
      Undefined => write!(f, "undefined"),
      Null => write!(f, "null"),
      Bool(b) => b.fmt(f),
      Number(x) => {
        if x.is_infinite() {
          if x.is_sign_positive() {
            write!(f, "Infinity")
          } else {
            write!(f, "-Infinity")
          }
        } else {
          x.fmt(f)
        }
      } // TODO: Match js's number string format
      BigInt(x) => x.fmt(f),
      Symbol(s) => write!(f, "Symbol(Symbol.{})", symbol_to_name(s.clone())),
      String(s) => s.fmt(f),
      Array(vals) => {
        if vals.elements.len() == 0 {
          Ok(())
        } else if vals.elements.len() == 1 {
          vals.elements[0].fmt(f)
        } else {
          let mut iter = vals.elements.iter();
          iter.next().unwrap().fmt(f)?;

          for val in iter {
            write!(f, ",")?;

            match val.typeof_() {
              VsType::Undefined => {}
              _ => {
                val.fmt(f)?;
              }
            };
          }

          Ok(())
        }
      }
      Object(_) => write!(f, "[object Object]"),
      Function(_) => write!(f, "[function]"),
      Class(_) => write!(f, "[class]"),
      Static(val) => val.fmt(f),
      Custom(val) => val.fmt(f),
    }
  }
}

pub trait ToVal {
  fn to_val(self) -> Val;
}

impl<T> From<T> for Val
where
  T: ToVal,
{
  fn from(value: T) -> Val {
    value.to_val()
  }
}

impl ToVal for char {
  fn to_val(self) -> Val {
    self.to_string().to_val()
  }
}

impl ToVal for &str {
  fn to_val(self) -> Val {
    Val::String(Rc::new(self.to_string()))
  }
}

impl ToVal for String {
  fn to_val(self) -> Val {
    Val::String(Rc::new(self))
  }
}

impl ToVal for f64 {
  fn to_val(self) -> Val {
    Val::Number(self)
  }
}

impl ToVal for bool {
  fn to_val(self) -> Val {
    Val::Bool(self)
  }
}

impl ToVal for BigInt {
  fn to_val(self) -> Val {
    Val::BigInt(self)
  }
}

impl ToVal for Vec<Val> {
  fn to_val(self) -> Val {
    Val::Array(Rc::new(VsArray::from(self)))
  }
}

pub struct PrettyVal<'a> {
  val: &'a Val,
}

impl<'a> Val {
  pub fn pretty(&'a self) -> PrettyVal<'a> {
    PrettyVal { val: self }
  }
}

impl<'a> std::fmt::Display for PrettyVal<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.val {
      Val::Void => write!(f, "void"),
      Val::Undefined => write!(f, "\x1b[90mundefined\x1b[39m"),
      Val::Null => write!(f, "\x1b[1mnull\x1b[22m"),
      Val::Bool(_) => write!(f, "\x1b[33m{}\x1b[39m", self.val),
      Val::Number(_) => write!(f, "\x1b[33m{}\x1b[39m", self.val),
      Val::BigInt(_) => write!(f, "\x1b[33m{}n\x1b[39m", self.val),
      Val::Symbol(_) => write!(f, "\x1b[32m{}\x1b[39m", self.val.codify()),
      Val::String(_) => write!(f, "\x1b[32m{}\x1b[39m", self.val.codify()),
      Val::Array(array) => {
        if array.elements.len() == 0 {
          return write!(f, "[]");
        }

        write!(f, "[ ")?;

        let mut first = true;

        for elem in &array.elements {
          if first {
            first = false;
          } else {
            write!(f, ", ")?;
          }

          write!(f, "{}", elem.pretty())?;
        }

        write!(f, " ]")
      }
      Val::Object(object) => {
        if let Some(proto) = &object.prototype {
          match op_sub(proto.clone(), "name".to_val()) {
            Ok(name) => {
              if name.typeof_() == VsType::String {
                write!(f, "{} ", name)?;
              }
            }
            Err(_) => {}
          }
        }

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
            write!(f, ", ")?;
          }

          write!(f, "{}: {}", k, v.pretty())?;
        }

        f.write_str(" }")
      }
      Val::Function(_) => write!(f, "\x1b[36m[Function]\x1b[39m"),
      Val::Class(_) => write!(f, "\x1b[36m[Class]\x1b[39m"),

      // TODO: Improve printing these
      Val::Static(s) => s.pretty_fmt(f),
      Val::Custom(c) => c.pretty_fmt(f),
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
