use core::fmt;
use std::any::Any;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::rc::Rc;
use std::str::FromStr;

use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;
use num_traits::Zero;

use crate::binary_op::BinaryOp;
use crate::copy_counter::CopyCounter;
use crate::native_function::ThisWrapper;
use crate::operations::{op_sub, op_submov};
use crate::stack_frame::StackFrame;
use crate::unary_op::UnaryOp;
use crate::vs_array::VsArray;
use crate::vs_class::VsClass;
use crate::vs_function::VsFunction;
use crate::vs_object::VsObject;
use crate::vs_storage_ptr::VsStoragePtr;
use crate::vs_symbol::{symbol_to_name, VsSymbol};

#[derive(Clone, Debug, Default)]
pub enum Val {
  #[default]
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
  StoragePtr(Rc<VsStoragePtr>),
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

  fn override_binary_op(
    &self,
    _op: BinaryOp,
    _left: &Val,
    _right: &Val,
  ) -> Option<Result<Val, Val>> {
    None
  }

  fn override_unary_op(&self, _op: UnaryOp) -> Option<Result<Val, Val>> {
    None
  }

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

  pub fn from_json(json: &serde_json::Value) -> Val {
    match json {
      serde_json::Value::Null => Val::Null,
      serde_json::Value::Bool(b) => Val::Bool(*b),
      serde_json::Value::Number(n) => {
        if let Some(n) = n.as_f64() {
          Val::Number(n)
        } else {
          // TODO: Is this possible? If so, handle without panicking.
          panic!("Non-f64 number")
        }
      }
      serde_json::Value::String(s) => s.clone().to_val(),
      serde_json::Value::Array(arr) => {
        let mut elements = Vec::new();

        for elem in arr {
          elements.push(Val::from_json(elem));
        }

        elements.to_val()
      }
      serde_json::Value::Object(obj) => {
        let mut string_map = BTreeMap::new();

        for (k, v) in obj {
          string_map.insert(k.clone(), Val::from_json(v));
        }

        VsObject {
          string_map,
          symbol_map: BTreeMap::new(),
          prototype: Val::Void,
        }
        .to_val()
      }
    }
  }

  pub fn to_json(&self) -> Option<serde_json::Value> {
    match self {
      Val::Void => Some(serde_json::Value::Null),
      Val::Undefined => Some(serde_json::Value::Null),
      Val::Null => Some(serde_json::Value::Null),
      Val::Bool(b) => Some(serde_json::Value::Bool(*b)),
      Val::Number(n) => Some(serde_json::Value::Number(serde_json::Number::from_f64(*n)?)),
      Val::BigInt(_) => None,
      Val::Symbol(_) => None,
      Val::String(s) => Some(serde_json::Value::String(s.to_string())),
      Val::Array(arr) => {
        let mut elements = Vec::new();

        for elem in &arr.elements {
          if let Some(elem) = elem.to_json() {
            elements.push(elem);
          } else {
            return None;
          }
        }

        Some(serde_json::Value::Array(elements))
      }
      Val::Object(obj) => {
        let mut string_map = serde_json::Map::new();

        for (k, v) in &obj.string_map {
          string_map.insert(k.clone(), v.to_json()?);
        }

        Some(serde_json::Value::Object(string_map))
      }
      Val::Function(_) => None,
      Val::Class(_) => None,
      Val::Static(_) => None,
      Val::Dynamic(_) => None,
      Val::CopyCounter(_) => None,
      Val::StoragePtr(ptr) => ptr.get().to_json(),
    }
  }

  pub fn not_ptr(&self) -> Val {
    match self {
      Val::StoragePtr(ptr) => ptr.get(),
      _ => self.clone(),
    }
  }
}

impl ValTrait for Val {
  fn typeof_(&self) -> VsType {
    use Val::*;

    match self {
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
      StoragePtr(ptr) => ptr.get().typeof_(),
    }
  }

  fn to_number(&self) -> f64 {
    use Val::*;

    match self {
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
      StoragePtr(ptr) => ptr.get().to_number(),
    }
  }

  fn to_index(&self) -> Option<usize> {
    use Val::*;

    match self {
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
      StoragePtr(ptr) => ptr.get().to_index(),
    }
  }

  fn is_primitive(&self) -> bool {
    use Val::*;

    match self {
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
      StoragePtr(ptr) => ptr.get().is_primitive(),
    }
  }

  fn is_truthy(&self) -> bool {
    use Val::*;

    match self {
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
      StoragePtr(ptr) => ptr.get().is_truthy(),
    }
  }

  fn is_nullish(&self) -> bool {
    use Val::*;

    match self {
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
      StoragePtr(ptr) => ptr.get().is_nullish(),
    }
  }

  fn bind(&self, params: Vec<Val>) -> Option<Val> {
    use Val::*;

    match self {
      Function(f) => Some(f.bind(params).to_val()),
      Static(val) => val.bind(params),
      Dynamic(val) => val.bind(params),

      _ => None,
    }
  }

  // TODO: &BigInt ?
  fn as_bigint_data(&self) -> Option<BigInt> {
    use Val::*;

    match self {
      BigInt(b) => Some(b.clone()),
      // TODO: Static? Others too?
      Dynamic(val) => val.as_bigint_data(),

      _ => None,
    }
  }

  fn as_array_data(&self) -> Option<Rc<VsArray>> {
    use Val::*;

    match self {
      Array(a) => Some(a.clone()),
      Dynamic(val) => val.as_array_data(),
      StoragePtr(ptr) => ptr.get().as_array_data(),

      _ => None,
    }
  }

  fn as_class_data(&self) -> Option<Rc<VsClass>> {
    use Val::*;

    match self {
      Class(class) => Some(class.clone()),
      Static(s) => s.as_class_data(),
      Dynamic(val) => val.as_class_data(),
      StoragePtr(ptr) => ptr.get().as_class_data(),

      _ => None,
    }
  }

  fn load_function(&self) -> LoadFunctionResult {
    use Val::*;

    match self {
      Function(f) => LoadFunctionResult::StackFrame(f.make_frame()),
      Static(s) => s.load_function(),
      Dynamic(val) => val.load_function(),
      StoragePtr(ptr) => ptr.get().load_function(),

      _ => LoadFunctionResult::NotAFunction,
    }
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
            return Some(matches!(
              key.to_string().as_str(),
              "at"
                | "concat"
                | "copyWithin"
                | "entries"
                | "every"
                | "fill"
                | "filter"
                | "find"
                | "findIndex"
                | "flat"
                | "flatMap"
                | "includes"
                | "indexOf"
                | "join"
                | "keys"
                | "lastIndexOf"
                | "length"
                | "map"
                | "pop"
                | "push"
                | "reduce"
                | "reduceRight"
                | "reverse"
                | "shift"
                | "slice"
                | "some"
                | "sort"
                | "splice"
                | "toLocaleString"
                | "toString"
                | "unshift"
                | "values"
            ));
          }
          Some(i) => i,
        };

        Some(index < array.elements.len())
      }
      Val::Object(object) => match key {
        Val::Symbol(symbol) => {
          if object.symbol_map.contains_key(symbol) {
            Some(true)
          } else {
            match &object.prototype {
              Val::Void => Some(false),
              prototype => prototype.has(key),
            }
          }
        }
        _ => {
          if object.string_map.contains_key(&key.to_string()) {
            Some(true)
          } else {
            match &object.prototype {
              Val::Void => Some(false),
              prototype => prototype.has(key),
            }
          }
        }
      },
      Val::Function(_) => Some(false),
      Val::Class(class) => class.static_.has(key),
      Val::Static(static_) => static_.has(key),
      Val::Dynamic(dynamic) => dynamic.has(key),
      Val::CopyCounter(_) => Some(matches!(key.to_string().as_str(), "tag" | "count")),
      Val::StoragePtr(ptr) => ptr.get().has(key),
    }
  }

  fn submov(&mut self, key: &Val, value: Val) -> Result<(), Val> {
    op_submov(self, key, value)
  }

  fn override_binary_op(&self, op: BinaryOp, left: &Val, right: &Val) -> Option<Result<Val, Val>> {
    match self {
      Val::Void
      | Val::Undefined
      | Val::Null
      | Val::Bool(_)
      | Val::Number(_)
      | Val::BigInt(_)
      | Val::Symbol(_)
      | Val::String(_)
      | Val::Array(_)
      | Val::Object(_)
      | Val::Function(_)
      | Val::Class(_)
      | Val::CopyCounter(_)
      | Val::StoragePtr(_) => None,
      Val::Static(static_) => static_.override_binary_op(op, left, right),
      Val::Dynamic(dynamic) => dynamic.override_binary_op(op, left, right),
    }
  }

  fn override_unary_op(&self, op: UnaryOp) -> Option<Result<Val, Val>> {
    match self {
      Val::Void
      | Val::Undefined
      | Val::Null
      | Val::Bool(_)
      | Val::Number(_)
      | Val::BigInt(_)
      | Val::Symbol(_)
      | Val::String(_)
      | Val::Array(_)
      | Val::Object(_)
      | Val::Function(_)
      | Val::Class(_)
      | Val::CopyCounter(_)
      | Val::StoragePtr(_) => None,
      Val::Static(static_) => static_.override_unary_op(op),
      Val::Dynamic(dynamic) => dynamic.override_unary_op(op),
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
        if vals.elements.is_empty() {
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

        if let Ok(name) = object.prototype.sub(&"name".to_val()) {
          if name.typeof_() == VsType::String {
            res += &name.to_string();
          }
        }

        if object.string_map.is_empty() {
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
      Val::StoragePtr(_) => todo!(),
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
        if vals.elements.is_empty() {
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
      StoragePtr(ptr) => ptr.get().fmt(f),
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
        if array.elements.is_empty() {
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
        if let Ok(name) = object.prototype.sub(&"name".to_val()) {
          if name.typeof_() == VsType::String {
            write!(f, "{} ", name)?;
          }
        }

        if object.string_map.is_empty() {
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

      Val::StoragePtr(ptr) => ptr.get().pretty_fmt(f),
    }
  }
}

pub fn number_to_index(x: f64) -> Option<usize> {
  if x < 0_f64 || x != x.floor() {
    return None;
  }

  Some(x as usize)
}

pub fn stringify_string(str: &str) -> String {
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
