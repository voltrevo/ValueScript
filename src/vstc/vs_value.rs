use std::rc::Rc;
use std::cell::RefCell;
use super::bytecode_decoder::BytecodeDecoder;
use super::bytecode_decoder::BytecodeType;

pub type Val = Rc<dyn VsValue>;

#[derive(PartialEq)]
enum VsType {
  Undefined,
  Null,
  Bool,
  Number,
  String,
  Array,
  Object,
  Function,
}

impl VsType {
  pub fn as_val(&self) -> Val {
    return VsString::from_str(match self {
      Undefined => "undefined",
      Null => "object",
      Bool => "boolean",
      Number => "number",
      String => "string",
      Array => "object",
      Object => "object",
      Function => "function",
    });
  }
}

pub trait VsValue {
  fn typeof_(&self) -> VsType;
  fn to_string(&self) -> String;
  fn to_number(&self) -> f64;
}

pub struct VsNumber {
  value: f64,
}

impl VsNumber {
  pub fn from_f64(value: f64) -> Val {
    return Rc::new(VsNumber { value: value });
  }
}

pub struct VsString {
  value: String,
}

impl VsString {
  pub fn from_str(value: &str) -> Val {
    return Rc::new(VsString { value: value.to_string() });
  }

  pub fn from_string(value: String) -> Val {
    return Rc::new(VsString { value: value });
  }
}

pub struct VsPointer {
  bytecode: Rc<Vec<u8>>,
  pos: usize,
  decoded: RefCell<Option<Val>>,
}

impl VsPointer {
  pub fn new(bytecode: &Rc<Vec<u8>>, from_pos: usize, pos: usize) -> Val {
    if pos < from_pos {
      let mut bd = BytecodeDecoder {
        data: bytecode.clone(),
        pos: pos,
      };

      let byte = bd.decode_byte();

      if byte != BytecodeType::Function as u8 {
        // Prevent circular objects
        std::panic!("Invalid: non-function pointer that points backwards");
      }
    }

    return Rc::new(VsPointer {
      bytecode: bytecode.clone(),
      pos: pos,
      decoded: RefCell::new(None),
    });
  }

  pub fn decode(&self) -> Val {
    let mut decoded = self.decoded.borrow_mut();

    if decoded.is_some() {
      return decoded.clone().unwrap();
    }

    let mut bd = BytecodeDecoder {
      data: self.bytecode.clone(),
      pos: self.pos,
    };

    let val = bd.decode_val();

    // TODO: Check that this actually inserts into the cell and not just a copy
    // somehow
    *decoded = Some(val.clone());

    return val;
  }
}

impl VsValue for VsNumber {
  fn typeof_(&self) -> VsType {
    return VsType::Number;
  }

  fn to_string(&self) -> String {
    return self.value.to_string();
  }

  fn to_number(&self) -> f64 {
    return self.value;
  }
}

impl VsValue for VsString {
  fn typeof_(&self) -> VsType {
    return VsType::String;
  }

  fn to_string(&self) -> String {
    return self.value.clone();
  }

  fn to_number(&self) -> f64 {
    std::panic!("not implemented");
  }
}

impl VsValue for VsPointer {
  fn typeof_(&self) -> VsType {
    let mut bd = BytecodeDecoder {
      data: self.bytecode.clone(),
      pos: self.pos,
    };

    return match bd.decode_type() {
      BytecodeType::Undefined => VsType::Undefined,
      BytecodeType::Null => VsType::Null,
      BytecodeType::False => VsType::Bool,
      BytecodeType::True => VsType::Bool,
      BytecodeType::SignedByte => VsType::Number,
      BytecodeType::Number => VsType::Number,
      BytecodeType::String => VsType::String,
      BytecodeType::Array => VsType::Array,
      BytecodeType::Object => VsType::Object,
      BytecodeType::Function => VsType::Function,
      BytecodeType::Instance => std::panic!("Not implemented"),
      BytecodeType::Pointer => std::panic!("Invalid: pointer to pointer"),
      BytecodeType::Register => std::panic!("Invalid: pointer to register"),
    }
  }

  fn to_string(&self) -> String {
    return self.decode().to_string();
  }

  fn to_number(&self) -> f64 {
    return self.decode().to_number();
  }
}

impl std::fmt::Display for dyn VsValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

pub fn add(left: &Rc<dyn VsValue>, right: &Rc<dyn VsValue>) -> Rc<dyn VsValue> {
  if left.typeof_() == VsType::String || right.typeof_() == VsType::String {
    return VsString::from_string(left.to_string() + &right.to_string());
  }

  return VsNumber::from_f64(left.to_number() + right.to_number());
}
