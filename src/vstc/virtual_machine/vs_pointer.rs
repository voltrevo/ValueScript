use std::rc::Rc;
use std::cell::RefCell;

use super::vs_value::Val;
use super::vs_value::ValTrait;
use super::vs_value::VsType;
use super::virtual_machine::StackFrame;
use super::bytecode_decoder::BytecodeDecoder;
use super::bytecode_decoder::BytecodeType;

pub struct VsPointer {
  bytecode: Rc<Vec<u8>>,
  pos: usize,
  decoded: RefCell<Option<Val>>,
}

impl VsPointer {
  pub fn new(bytecode: &Rc<Vec<u8>>, pos: usize) -> Val {
    return Val::Custom(Rc::new(VsPointer {
      bytecode: bytecode.clone(),
      pos: pos,
      decoded: RefCell::new(None),
    }));
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

    let val = bd.decode_val(&Vec::new());

    // TODO: Check that this actually inserts into the cell and not just a copy
    // somehow
    *decoded = Some(val.clone());

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

  fn val_to_string(&self) -> String {
    return self.decode().val_to_string();
  }

  fn to_number(&self) -> f64 {
    return self.decode().to_number();
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
    return self.decode().to_primitive();
  }

  fn make_frame(&self) -> Option<StackFrame> {
    return self.decode().make_frame();
  }

  fn is_truthy(&self) -> bool {
    return self.decode().is_truthy();
  }
}
