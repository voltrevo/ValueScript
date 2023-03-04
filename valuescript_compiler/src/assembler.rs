use std::collections::{HashMap, HashSet};

use crate::asm::{
  Array, Builtin, Class, Definition, DefinitionContent, DefinitionRef, Function, Instruction,
  InstructionOrLabel, Label, LabelRef, Module, Object, Register, Value,
};

pub fn assemble(assembly: &Module) -> Vec<u8> {
  let mut assembler = Assembler {
    output: Vec::new(),
    fn_data: Default::default(),
    definitions_map: LocationMap {
      references: HashMap::new(),
      found_locations: HashMap::new(),
    },
  };

  assembler.module(assembly);

  return assembler.output;
}

struct Assembler {
  output: Vec<u8>,
  fn_data: AssemblerFnData,
  definitions_map: LocationMap,
}

impl Assembler {
  fn module(&mut self, module: &Module) {
    for definition in &module.definitions {
      self.definition(definition);
    }

    self.definitions_map.resolve(&mut self.output);
  }

  fn definition(&mut self, definition: &Definition) {
    self.definitions_map.found_locations.insert(
      LocationRef::DefinitionRef(definition.ref_.clone()),
      self.output.len(),
    );

    match &definition.content {
      DefinitionContent::Function(function) => {
        self.function(function);
      }
      DefinitionContent::Class(class) => {
        self.class(class);
      }
      DefinitionContent::Value(value) => {
        self.value(value);
      }
    }
  }

  fn function(&mut self, function: &Function) {
    self.output.push(ValueType::Function as u8);

    self.fn_data = Default::default();

    self.fn_data.register_count_pos = self.output.len();
    self.output.push(0xff); // Placeholder for register count

    self.output.push(function.parameters.len() as u8);

    let mut param_set = HashSet::<Register>::new();

    for parameter in &function.parameters {
      let inserted = param_set.insert(parameter.clone());

      if !inserted {
        panic!("Duplicate parameter: {}", parameter);
      }

      self.register(parameter);
    }

    for instruction_or_label in &function.body {
      match instruction_or_label {
        InstructionOrLabel::Instruction(instruction) => {
          self.instruction(instruction);
        }
        InstructionOrLabel::Label(label) => {
          self.label(label);
        }
      }
    }

    self.output.push(Instruction::End.byte());

    // TODO: Handle >255 registers
    self.output[self.fn_data.register_count_pos] = self.fn_data.register_map.len() as u8;

    self.fn_data.labels_map.resolve(&mut self.output);
  }

  fn class(&mut self, class: &Class) {
    self.output.push(ValueType::Class as u8);
    self.value(&class.constructor);
    self.value(&class.methods);
  }

  fn label(&mut self, label: &Label) {
    self
      .fn_data
      .labels_map
      .found_locations
      .insert(LocationRef::LabelRef(label.ref_()), self.output.len());
  }

  fn instruction(&mut self, instruction: &Instruction) {
    use Instruction::*;

    self.output.push(instruction.byte());

    match instruction {
      End => {}
      OpInc(dst) | OpDec(dst) => {
        self.register(dst);
      }
      Mov(arg, dst)
      | OpNot(arg, dst)
      | OpBitNot(arg, dst)
      | TypeOf(arg, dst)
      | UnaryPlus(arg, dst)
      | UnaryMinus(arg, dst) => {
        self.value(arg);
        self.register(dst);
      }
      OpPlus(arg1, arg2, dst)
      | OpMinus(arg1, arg2, dst)
      | OpMul(arg1, arg2, dst)
      | OpDiv(arg1, arg2, dst)
      | OpMod(arg1, arg2, dst)
      | OpExp(arg1, arg2, dst)
      | OpEq(arg1, arg2, dst)
      | OpNe(arg1, arg2, dst)
      | OpTripleEq(arg1, arg2, dst)
      | OpTripleNe(arg1, arg2, dst)
      | OpAnd(arg1, arg2, dst)
      | OpOr(arg1, arg2, dst)
      | OpLess(arg1, arg2, dst)
      | OpLessEq(arg1, arg2, dst)
      | OpGreater(arg1, arg2, dst)
      | OpGreaterEq(arg1, arg2, dst)
      | OpNullishCoalesce(arg1, arg2, dst)
      | OpOptionalChain(arg1, arg2, dst)
      | OpBitAnd(arg1, arg2, dst)
      | OpBitOr(arg1, arg2, dst)
      | OpBitXor(arg1, arg2, dst)
      | OpLeftShift(arg1, arg2, dst)
      | OpRightShift(arg1, arg2, dst)
      | OpRightShiftUnsigned(arg1, arg2, dst)
      | InstanceOf(arg1, arg2, dst)
      | In(arg1, arg2, dst)
      | Call(arg1, arg2, dst)
      | Bind(arg1, arg2, dst)
      | Sub(arg1, arg2, dst)
      | SubMov(arg1, arg2, dst)
      | New(arg1, arg2, dst) => {
        self.value(arg1);
        self.value(arg2);
        self.register(dst);
      }
      Apply(arg1, arg2, arg3, dst) | SubCall(arg1, arg2, arg3, dst) => {
        self.value(arg1);
        self.value(arg2);
        self.value(arg3);
        self.register(dst);
      }
      Jmp(label_ref) => {
        self.label_ref(label_ref);
      }
      JmpIf(value, label_ref) => {
        self.value(value);
        self.label_ref(label_ref);
      }
    }
  }

  fn value(&mut self, value: &Value) {
    match value {
      Value::Register(register) => {
        self.output.push(ValueType::Register as u8);
        self.register(register);
      }
      Value::Number(number) => self.number(*number),
      Value::String(string) => self.string(string),
      Value::Bool(boolean) => match boolean {
        false => self.output.push(ValueType::False as u8),
        true => self.output.push(ValueType::True as u8),
      },
      Value::Void => self.output.push(ValueType::Void as u8),
      Value::Null => self.output.push(ValueType::Null as u8),
      Value::Undefined => self.output.push(ValueType::Undefined as u8),
      Value::Array(array) => self.array(array),
      Value::Object(object) => self.object(object),
      Value::DefinitionRef(definition_ref) => self.definition_ref(definition_ref),
      Value::Builtin(builtin) => self.builtin(builtin),
    }
  }

  fn label_ref(&mut self, label_ref: &LabelRef) {
    self
      .fn_data
      .labels_map
      .add_unresolved(LocationRef::LabelRef(label_ref.clone()), &mut self.output);
  }

  fn register(&mut self, register: &Register) {
    let reg_index = self.lookup_register(register);
    self.output.push(reg_index);
  }

  fn lookup_register(&mut self, register: &Register) -> u8 {
    match register {
      Register::Return => 0,
      Register::This => 1,
      Register::Named(_) => match self.fn_data.register_map.get(register) {
        Some(index) => *index,
        None => {
          // TODO: Support >255 registers
          let index = (self.fn_data.register_map.len() as u8) + 2;
          self.fn_data.register_map.insert(register.clone(), index);
          index
        }
      },
      Register::Ignore => 0xff,
    }
  }

  fn varsize_uint(&mut self, value: usize) {
    let mut x = value;

    loop {
      let mut b: u8 = (x % 128) as u8;
      x /= 128;

      if x != 0 {
        b += 128;
      }

      self.output.push(b);

      if x == 0 {
        break;
      }
    }
  }

  fn number(&mut self, value: f64) {
    self.output.push(ValueType::Number as u8);

    if value == (value as i8) as f64 {
      self.output.push(ValueType::SignedByte as u8);

      for b in (value as i8).to_le_bytes() {
        self.output.push(b);
      }
    } else {
      self.output.push(ValueType::Number as u8);

      for b in value.to_le_bytes() {
        self.output.push(b);
      }
    }
  }

  fn string(&mut self, value: &String) {
    self.output.push(ValueType::String as u8);
    self.varsize_uint(value.len());

    for b in value.as_bytes() {
      self.output.push(*b);
    }
  }

  fn definition_ref(&mut self, value: &DefinitionRef) {
    self.output.push(ValueType::Function as u8);
    self
      .definitions_map
      .add_unresolved(LocationRef::DefinitionRef(value.clone()), &mut self.output);
  }

  fn builtin(&mut self, builtin: &Builtin) {
    self.output.push(ValueType::Builtin as u8);

    let builtin_code = match builtin.name.as_str() {
      "Math" => 0,
      "Debug" => 1,
      _ => panic!("Unknown builtin: {}", builtin.name),
    };

    self.varsize_uint(builtin_code);
  }

  fn array(&mut self, array: &Array) {
    self.output.push(ValueType::Array as u8);
    self.varsize_uint(array.values.len());

    for value in &array.values {
      self.value(value);
    }
  }

  fn object(&mut self, object: &Object) {
    self.output.push(ValueType::Object as u8);
    self.varsize_uint(object.properties.len());

    for (key, value) in &object.properties {
      self.value(key);
      self.value(value);
    }
  }
}

enum ValueType {
  End = 0x00,
  Void = 0x01,
  Undefined = 0x02,
  Null = 0x03,
  False = 0x04,
  True = 0x05,
  SignedByte = 0x06,
  Number = 0x07,
  String = 0x08,
  Array = 0x09,
  Object = 0x0a,
  Function = 0x0b,
  // Instance = 0x0c,
  Pointer = 0x0d,
  Register = 0x0e,
  // External = 0x0f,
  Builtin = 0x10,
  Class = 0x11,
}

#[derive(Hash, PartialEq, Eq, Clone)]
enum LocationRef {
  DefinitionRef(DefinitionRef),
  LabelRef(LabelRef),
}

impl std::fmt::Display for LocationRef {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      LocationRef::DefinitionRef(def) => write!(f, "{}", def),
      LocationRef::LabelRef(label) => write!(f, "{}", label),
    }
  }
}

#[derive(Default)]
struct LocationMap {
  references: HashMap<LocationRef, Vec<usize>>,
  found_locations: HashMap<LocationRef, usize>,
}

impl LocationMap {
  fn add_unresolved(&mut self, ref_: LocationRef, output: &mut Vec<u8>) {
    self.references.entry(ref_).or_default().push(output.len());

    output.push(0xff);
    output.push(0xff); // TODO: Support >65535
  }

  fn resolve(&self, output: &mut Vec<u8>) {
    for (name, ref_locations) in &self.references {
      let location_optional = self.found_locations.get(name);

      if location_optional.is_none() {
        std::panic!("Unresolved reference to {} at {}", name, ref_locations[0]);
      }

      let location = location_optional.unwrap();

      for ref_location in ref_locations {
        output[*ref_location] = (*location % 256) as u8;
        output[*ref_location + 1] = (*location / 256) as u8; // TODO: Support >65535
      }
    }
  }
}

#[derive(Default)]
struct AssemblerFnData {
  register_map: HashMap<Register, u8>,
  register_count_pos: usize,
  labels_map: LocationMap,
}
