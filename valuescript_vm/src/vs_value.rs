use core::fmt;
use std::any::Any;
use std::fmt::Display;
use std::rc::Rc;
use std::str::FromStr;

use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;
use num_traits::Zero;

use crate::copy_counter::CopyCounter;
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
  String(Rc<str>),
  Array(Rc<VsArray>),
  Object(Rc<VsObject>),
  Function(Rc<VsFunction>),
  Class(Rc<VsClass>),
  Static(&'static dyn ValTrait),
  Dynamic(Rc<dyn DynValTrait>),
  CopyCounter(Box<CopyCounter>),
}

impl Default for Val {
  fn default() -> Self {
    Val::Void
  }
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

impl Display for VsType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      VsType::Undefined => f.write_str("undefined"),
      VsType::Null => f.write_str("null"),
      VsType::Bool => f.write_str("bool"),
      VsType::Number => f.write_str("number"),
      VsType::BigInt => f.write_str("bigint"),
      VsType::Symbol => f.write_str("symbol"),
      VsType::String => f.write_str("string"),
      VsType::Array => f.write_str("array"),
      VsType::Object => f.write_str("object"),
      VsType::Function => f.write_str("function"),
      VsType::Class => f.write_str("class"),
    }
  }
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
  fn as_class_data(&self) -> Option<Rc<VsClass>>;

  fn load_function(&self) -> LoadFunctionResult;

  fn sub(&self, key: &Val) -> Result<Val, Val>;
  fn has(&self, key: &Val) -> Option<bool>;
  fn submov(&mut self, key: &Val, value: Val) -> Result<(), Val>;

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
  fn codify(&self) -> String;
}

pub trait DynValTrait: ValTrait {
  fn clone_interior(&self) -> Rc<dyn DynValTrait>;
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> DynValTrait for T
where
  T: 'static + ValTrait + Clone,
{
  fn clone_interior(&self) -> Rc<dyn DynValTrait> {
    Rc::new(self.clone())
  }

  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }
}

pub fn dynamic_make_mut(rc: &mut Rc<dyn DynValTrait>) -> &mut dyn DynValTrait {
  if Rc::get_mut(rc).is_none() {
    *rc = rc.clone_interior();
  }

  Rc::get_mut(rc).unwrap()
}

impl fmt::Debug for dyn ValTrait {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "(dyn ValTrait)(")?;
    self.pretty_fmt(f)?;
    write!(f, ")")
  }
}

impl fmt::Debug for dyn DynValTrait {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "(dyn DynValTrait)(")?;
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
      Dynamic(val) => val.typeof_(),
      CopyCounter(_) => VsType::Object,
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
      Dynamic(val) => val.to_number(),
      CopyCounter(_) => f64::NAN,
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
      Dynamic(val) => val.to_index(),
      CopyCounter(_) => None,
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
      Dynamic(val) => val.is_primitive(),
      CopyCounter(_) => false,
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
      Dynamic(val) => val.is_truthy(),
      CopyCounter(_) => true,
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
      Dynamic(val) => val.is_nullish(),
      CopyCounter(_) => false,
    };
  }

  fn bind(&self, params: Vec<Val>) -> Option<Val> {
    use Val::*;

    return match self {
      Function(f) => Some(f.bind(params).to_val()),
      Static(val) => val.bind(params),
      Dynamic(val) => val.bind(params),

      _ => None,
    };
  }

  // TODO: &BigInt ?
  fn as_bigint_data(&self) -> Option<BigInt> {
    use Val::*;

    return match self {
      BigInt(b) => Some(b.clone()),
      // TODO: Static? Others too?
      Dynamic(val) => val.as_bigint_data(),

      _ => None,
    };
  }

  fn as_array_data(&self) -> Option<Rc<VsArray>> {
    use Val::*;

    return match self {
      Array(a) => Some(a.clone()),
      Dynamic(val) => val.as_array_data(),

      _ => None,
    };
  }

  fn as_class_data(&self) -> Option<Rc<VsClass>> {
    use Val::*;

    return match self {
      Class(class) => Some(class.clone()),
      Static(s) => s.as_class_data(),
      Dynamic(val) => val.as_class_data(),

      _ => None,
    };
  }

  fn load_function(&self) -> LoadFunctionResult {
    use Val::*;

    return match self {
      Function(f) => LoadFunctionResult::StackFrame(f.make_frame()),
      Static(s) => s.load_function(),
      Dynamic(val) => val.load_function(),

      _ => LoadFunctionResult::NotAFunction,
    };
  }

  fn sub(&self, key: &Val) -> Result<Val, Val> {
    // TODO: mut version?
    op_sub(&mut self.clone(), key)
  }

  fn has(&self, key: &Val) -> Option<bool> {
    match self {
      Val::Void
      | Val::Undefined
      | Val::Null
      | Val::Bool(_)
      | Val::Number(_)
      | Val::BigInt(_)
      | Val::Symbol(_)
      | Val::String(_) => None,

      Val::Array(array) => {
        let index = match key.to_index() {
          None => {
            return Some(match key.to_string().as_str() {
              "at" => true,
              "concat" => true,
              "copyWithin" => true,
              "entries" => true,
              "every" => true,
              "fill" => true,
              "filter" => true,
              "find" => true,
              "findIndex" => true,
              "flat" => true,
              "flatMap" => true,
              "includes" => true,
              "indexOf" => true,
              "join" => true,
              "keys" => true,
              "lastIndexOf" => true,
              "length" => true,
              "map" => true,
              "pop" => true,
              "push" => true,
              "reduce" => true,
              "reduceRight" => true,
              "reverse" => true,
              "shift" => true,
              "slice" => true,
              "some" => true,
              "sort" => true,
              "splice" => true,
              "toLocaleString" => true,
              "toString" => true,
              "unshift" => true,
              "values" => true,

              _ => false,
            });
          }
          Some(i) => i,
        };

        return Some(index < array.elements.len());
      }
      Val::Object(object) => match key {
        Val::Symbol(symbol) => {
          if object.symbol_map.contains_key(symbol) {
            return Some(true);
          }

          if let Some(proto) = &object.prototype {
            return proto.has(key);
          }

          return Some(false);
        }
        _ => {
          if object.string_map.contains_key(&key.to_string()) {
            return Some(true);
          }

          if let Some(proto) = &object.prototype {
            return proto.has(key);
          }

          return Some(false);
        }
      },
      Val::Function(_) => Some(false),
      Val::Class(class) => class.static_.has(key),
      Val::Static(static_) => static_.has(key),
      Val::Dynamic(dynamic) => dynamic.has(key),
      Val::CopyCounter(_) => Some(match key.to_string().as_str() {
        "tag" | "count" => true,
        _ => false,
      }),
    }
  }

  fn submov(&mut self, key: &Val, value: Val) -> Result<(), Val> {
    op_submov(self, key, value)
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
          match proto.sub(&"name".to_val()) {
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
      Val::Dynamic(val) => val.codify(),
      Val::CopyCounter(cc) => format!(
        "CopyCounter {{ tag: {}, count: {} }}",
        cc.tag.codify(),
        cc.count.borrow()
      ),
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
      Dynamic(val) => val.fmt(f),
      CopyCounter(cc) => write!(
        f,
        "CopyCounter {{ tag: {}, count: {} }}",
        cc.tag,
        (*cc.count.borrow() as f64).to_val()
      ),
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
    Val::String(Rc::from(self.to_owned()))
  }
}

impl ToVal for String {
  fn to_val(self) -> Val {
    Val::String(Rc::from(self))
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

impl<T> ToVal for &'static T
where
  T: ValTrait,
{
  fn to_val(self) -> Val {
    Val::Static(self)
  }
}

pub trait ToDynamicVal {
  fn to_dynamic_val(self) -> Val;
}

impl<T> ToDynamicVal for T
where
  T: DynValTrait + 'static,
{
  fn to_dynamic_val(self) -> Val {
    Val::Dynamic(Rc::new(self))
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
          match proto.sub(&"name".to_val()) {
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
      Val::Dynamic(c) => c.pretty_fmt(f),

      Val::CopyCounter(cc) => write!(
        f,
        "\x1b[36mCopyCounter\x1b[39m {{ tag: {}, count: {} }}",
        cc.tag.pretty(),
        (*cc.count.borrow() as f64).to_val().pretty()
      ),
    }
  }
}

pub fn number_to_index(x: f64) -> Option<usize> {
  if x < 0_f64 || x != x.floor() {
    return None;
  }

  return Some(x as usize);
}

fn stringify_string(str: &str) -> String {
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
