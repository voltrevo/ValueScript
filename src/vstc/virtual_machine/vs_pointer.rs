use std::rc::Rc;
use std::cell::RefCell;
use std::collections::BTreeMap;

use super::vs_value::Val;
use super::vs_value::ValTrait;
use super::vs_value::VsType;
use super::virtual_machine::StackFrame;
use super::bytecode_decoder::BytecodeDecoder;
use super::bytecode_decoder::BytecodeType;

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
    }
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
      Undefined => true,
      Null => true,
      Bool => true,
      Number => true,
      String => true,
      Array => false,
      Object => false,
      Function => false,
    }
  }

  fn to_primitive(&self) -> Val {
    return self.resolve().to_primitive();
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

  fn bind(&self, params: Vec<Val>) -> Option<Val> {
    return self.resolve().bind(params);
  }

  fn is_truthy(&self) -> bool {
    return self.resolve().is_truthy();
  }

  fn is_nullish(&self) -> bool {
    return self.resolve().is_nullish();
  }

  fn as_array_data(&self) -> Option<Rc<Vec<Val>>> {
    return self.resolve().as_array_data();
  }

  fn as_object_data(&self) -> Option<Rc<BTreeMap<String, Val>>> {
    return self.resolve().as_object_data();
  }

  fn make_frame(&self) -> Option<StackFrame> {
    return self.resolve().make_frame();
  }
}
