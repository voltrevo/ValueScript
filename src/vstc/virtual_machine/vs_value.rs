use std::rc::Rc;
use std::str::FromStr;

use super::vs_function::VsFunction;
use super::virtual_machine::StackFrame;
use super::vs_object::VsObject;
use super::vs_array::VsArray;

#[derive(Clone)]
pub enum Val {
  Void,
  Undefined,
  Null,
  Bool(bool),
  Number(f64),
  String(Rc<String>),
  Array(Rc<VsArray>),
  Object(Rc<VsObject>),
  Function(Rc<VsFunction>),
  Custom(Rc<dyn ValTrait>),
}

#[derive(PartialEq)]
pub enum VsType {
  Undefined,
  Null,
  Bool,
  Number,
  String,
  Array,
  Object,
  Function,
}

pub enum LoadFunctionResult {
  NotAFunction,
  StackFrame(StackFrame),
  NativeFunction(fn(this: &mut Val, params: Vec<Val>) -> Val),
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

  fn resolve(&self) -> Val;

  fn bind(&self, params: Vec<Val>) -> Option<Val>;

  fn as_array_data(&self) -> Option<Rc<VsArray>>;
  fn as_object_data(&self) -> Option<Rc<VsObject>>;

  fn load_function(&self) -> LoadFunctionResult;
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
      String(_) => VsType::String,
      Array(_) => VsType::Array,
      Object(_) => VsType::Object,
      Function(_) => VsType::Function,
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
      Number(x) => x.to_string(), // TODO: Match js's number string format
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
            res += val.val_to_string().as_str();
          }

          res
        }
      },
      Object(_) => "[object Object]".to_string(),
      Function(_) => "[function]".to_string(),
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
      String(s) => f64::from_str(s).unwrap_or(f64::NAN),
      Array(vals) => match vals.elements.len() {
        0 => 0_f64,
        1 => vals.elements[0].to_number(),
        _ => f64::NAN,
      },
      Object(_) => f64::NAN,
      Function(_) => f64::NAN,
      Custom(val) => val.to_number(),
    };
  }

  fn to_index(&self) -> Option<usize> {
    use Val::*;

    return match self {
      Void => std::panic!("Shouldn't happen"),
      Undefined => None,
      Null => None,
      Bool(_) => None,
      Number(x) => number_to_index(*x),
      String(s) => match f64::from_str(s) {
        Ok(x) => number_to_index(x),
        Err(_) => None,
      },
      Array(vals) => None,
      Object(_) => None,
      Function(_) => None,
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
      String(_) => true,
      Array(_) => false,
      Object(_) => false,
      Function(_) => false,
      Custom(val) => val.is_primitive(),
    }
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
      Number(x) => *x != 0_f64,
      String(s) => s.len() > 0,
      Array(_) => true,
      Object(_) => true,
      Function(_) => true,
      Custom(val) => val.is_truthy(),
    };
  }

  fn is_nullish(&self) -> bool {
    use Val::*;

    return match self {
      Void => std::panic!("Shouldn't happen"), // TODO: Or just true?
      Undefined => true,
      Null => true,
      Bool(_) => false,
      Number(_) => false,
      String(_) => false,
      Array(_) => false,
      Object(_) => false,
      Function(_) => false,
      Custom(val) => val.is_nullish(),
    };
  }

  fn resolve(&self) -> Val {
    std::panic!("Unexpected resolve call on plain Val")
  }

  fn bind(&self, params: Vec<Val>) -> Option<Val> {
    use Val::*;

    return match self {
      Function(f) => Some(Val::Function(Rc::new(f.bind(params)))),
      Custom(val) => val.bind(params),

      _ => None,
    }
  }

  fn as_array_data(&self) -> Option<Rc<VsArray>> {
    use Val::*;

    return match self {
      Array(a) => Some(a.clone()),
      Custom(val) => val.as_array_data(),

      _ => None,
    }
  }

  fn as_object_data(&self) -> Option<Rc<VsObject>> {
    use Val::*;

    return match self {
      Object(obj) => Some(obj.clone()),
      Custom(val) => val.as_object_data(),

      _ => None,
    }
  }

  fn load_function(&self) -> LoadFunctionResult {
    use Val::*;

    return match self {
      Function(f) => LoadFunctionResult::StackFrame(f.make_frame()),
      Custom(val) => val.load_function(),

      _ => LoadFunctionResult::NotAFunction,
    }
  }
}

impl std::fmt::Display for Val {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.val_to_string())
  }
}

fn number_to_index(x: f64) -> Option<usize> {
  if x < 0_f64 || x != x.floor() {
    return None
  }

  return Some(x as usize);
}
