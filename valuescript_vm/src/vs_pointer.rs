use std::cell::RefCell;
use std::rc::Rc;

use num_bigint::BigInt;

use super::bytecode_decoder::{BytecodeDecoder, BytecodeType};
use super::vs_array::VsArray;
use super::vs_class::VsClass;
use super::vs_object::VsObject;
use super::vs_value::{LoadFunctionResult, Val, ValTrait, VsType};

pub struct VsPointer {
  bytecode: Rc<Vec<u8>>,
  pos: usize,
  resolved: RefCell<Option<Val>>,
}

impl VsPointer {
  pub fn new(bytecode: &Rc<Vec<u8>>, pos: usize) -> Val {
    return Val::Custom(Rc::new(VsPointer {
      bytecode: bytecode.clone(),
      pos: pos,
      resolved: RefCell::new(None),
    }));
  }

  fn resolve(&self) -> Val {
    let mut resolved = self.resolved.borrow_mut();

    if resolved.is_some() {
      return resolved.clone().unwrap();
    }

    let mut bd = BytecodeDecoder {
      data: self.bytecode.clone(),
      pos: self.pos,
    };

    let val = bd.decode_val(&Vec::new());

    // TODO: Check that this actually inserts into the cell and not just a copy
    // somehow
    *resolved = Some(val.clone());

    return val;
  }
}

impl ValTrait for VsPointer {
  fn typeof_(&self) -> VsType {
    let mut bd = BytecodeDecoder {
      data: self.bytecode.clone(),
      pos: self.pos,
    };

    return match bd.decode_type() {
      BytecodeType::End => std::panic!("Invalid: pointer to end"),
      BytecodeType::Void => std::panic!("Invalid: pointer to void"),
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
      BytecodeType::Pointer => std::panic!("Invalid: pointer to pointer"),
      BytecodeType::Register => std::panic!("Invalid: pointer to register"),
      BytecodeType::Builtin => std::panic!("Invalid: pointer to builtin"),
      BytecodeType::Class => VsType::Class,
      BytecodeType::BigInt => VsType::BigInt,
      BytecodeType::Unrecognized => std::panic!("Unrecognized bytecode type at {}", self.pos - 1),
    };
  }

  fn val_to_string(&self) -> String {
    return self.resolve().val_to_string();
  }

  fn to_number(&self) -> f64 {
    return self.resolve().to_number();
  }

  fn to_index(&self) -> Option<usize> {
    return self.resolve().to_index();
  }

  fn is_primitive(&self) -> bool {
    return match self.typeof_() {
      VsType::Undefined => true,
      VsType::Null => true,
      VsType::Bool => true,
      VsType::Number => true,
      VsType::BigInt => true,
      VsType::String => true,
      VsType::Array => false,
      VsType::Object => false,
      VsType::Function => false,
      VsType::Class => false,
    };
  }

  fn to_primitive(&self) -> Val {
    return self.resolve().to_primitive();
  }

  fn bind(&self, params: Vec<Val>) -> Option<Val> {
    return self.resolve().bind(params);
  }

  fn is_truthy(&self) -> bool {
    return self.resolve().is_truthy();
  }

  fn is_nullish(&self) -> bool {
    return self.resolve().is_nullish();
  }

  fn as_bigint_data(&self) -> Option<BigInt> {
    return self.resolve().as_bigint_data();
  }

  fn as_array_data(&self) -> Option<Rc<VsArray>> {
    return self.resolve().as_array_data();
  }

  fn as_object_data(&self) -> Option<Rc<VsObject>> {
    return self.resolve().as_object_data();
  }

  fn as_class_data(&self) -> Option<Rc<VsClass>> {
    return self.resolve().as_class_data();
  }

  fn load_function(&self) -> LoadFunctionResult {
    return self.resolve().load_function();
  }

  fn sub(&self, subscript: Val) -> Val {
    return self.resolve().sub(subscript);
  }

  fn submov(&mut self, _subscript: Val, _value: Val) {
    std::panic!("Not implemented");
  }

  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.resolve().fmt(f)
  }

  fn codify(&self) -> String {
    self.resolve().codify()
  }
}
