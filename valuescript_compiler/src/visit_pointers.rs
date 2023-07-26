use crate::asm::{
  Array, Definition, DefinitionContent, ExportStar, FnLine, Instruction, Module, Object, Pointer,
  Value,
};

pub fn visit_pointers<Visitor>(module: &mut Module, visitor: Visitor)
where
  Visitor: FnMut(PointerVisitation),
{
  let mut pointer_visitor = VisitPointerImpl::new(visitor);
  pointer_visitor.module(module);
}

#[derive(PartialEq, Debug)]
pub enum PointerVisitation<'a> {
  Export(&'a mut Pointer),
  Definition(&'a mut Pointer),
  Reference(&'a Pointer, &'a mut Pointer),
}

struct VisitPointerImpl<Visitor>
where
  Visitor: FnMut(PointerVisitation),
{
  visitor: Visitor,
}

impl<Visitor> VisitPointerImpl<Visitor>
where
  Visitor: FnMut(PointerVisitation),
{
  fn new(visitor: Visitor) -> Self {
    Self { visitor }
  }

  pub fn module(&mut self, module: &mut Module) {
    self.value(None, &mut module.export_default);
    self.export_star(None, &mut module.export_star);

    for definition in &mut module.definitions {
      self.definition(definition);
    }
  }

  fn definition(&mut self, definition: &mut Definition) {
    (self.visitor)(PointerVisitation::Definition(&mut definition.pointer));

    match &mut definition.content {
      DefinitionContent::Function(function) => {
        self.body(&definition.pointer, &mut function.body);
      }
      DefinitionContent::Value(value) => {
        self.value(Some(&definition.pointer), value);
      }
      DefinitionContent::Lazy(lazy) => {
        self.body(&definition.pointer, &mut lazy.body);
      }
    }
  }

  fn array(&mut self, owner: Option<&Pointer>, array: &mut Array) {
    for value in &mut array.values {
      self.value(owner, value);
    }
  }

  fn object(&mut self, owner: Option<&Pointer>, object: &mut Object) {
    for (key, value) in object.properties.iter_mut() {
      self.value(owner, key);
      self.value(owner, value);
    }
  }

  fn export_star(&mut self, owner: Option<&Pointer>, export_star: &mut ExportStar) {
    for p in &mut export_star.includes {
      self.pointer(owner, p);
    }

    self.object(owner, &mut export_star.local);
  }

  fn value(&mut self, owner: Option<&Pointer>, value: &mut Value) {
    use Value::*;

    match value {
      Void | Undefined | Null | Bool(_) | Number(_) | BigInt(_) | String(_) | Register(_)
      | Builtin(_) => {}
      Array(array) => {
        self.array(owner, array);
      }
      Object(object) => {
        self.object(owner, object);
      }
      Class(class) => {
        self.value(owner, &mut class.constructor);
        self.value(owner, &mut class.prototype);
        self.value(owner, &mut class.static_);
      }
      Pointer(pointer) => {
        self.pointer(owner, pointer);
      }
    }
  }

  fn pointer(&mut self, owner: Option<&Pointer>, pointer: &mut Pointer) {
    (self.visitor)(match owner {
      Some(owner) => PointerVisitation::Reference(owner, pointer),
      None => PointerVisitation::Export(pointer),
    });
  }

  fn instruction(&mut self, owner: &Pointer, instruction: &mut Instruction) {
    use Instruction::*;

    match instruction {
      End | UnsetCatch | RequireMutableThis | OpInc(..) | OpDec(..) | Jmp(..) | SetCatch(..)
      | Next(..) | UnpackIterRes(..) => {}
      Mov(arg, _)
      | OpNot(arg, _)
      | OpBitNot(arg, _)
      | TypeOf(arg, _)
      | UnaryPlus(arg, _)
      | UnaryMinus(arg, _)
      | Import(arg, _)
      | ImportStar(arg, _)
      | Throw(arg)
      | Cat(arg, _)
      | Yield(arg, _)
      | YieldStar(arg, _) => {
        self.value(Some(owner), arg);
      }
      OpPlus(arg1, arg2, _)
      | OpMinus(arg1, arg2, _)
      | OpMul(arg1, arg2, _)
      | OpDiv(arg1, arg2, _)
      | OpMod(arg1, arg2, _)
      | OpExp(arg1, arg2, _)
      | OpEq(arg1, arg2, _)
      | OpNe(arg1, arg2, _)
      | OpTripleEq(arg1, arg2, _)
      | OpTripleNe(arg1, arg2, _)
      | OpAnd(arg1, arg2, _)
      | OpOr(arg1, arg2, _)
      | OpLess(arg1, arg2, _)
      | OpLessEq(arg1, arg2, _)
      | OpGreater(arg1, arg2, _)
      | OpGreaterEq(arg1, arg2, _)
      | OpNullishCoalesce(arg1, arg2, _)
      | OpOptionalChain(arg1, arg2, _)
      | OpBitAnd(arg1, arg2, _)
      | OpBitOr(arg1, arg2, _)
      | OpBitXor(arg1, arg2, _)
      | OpLeftShift(arg1, arg2, _)
      | OpRightShift(arg1, arg2, _)
      | OpRightShiftUnsigned(arg1, arg2, _)
      | InstanceOf(arg1, arg2, _)
      | In(arg1, arg2, _)
      | Call(arg1, arg2, _)
      | Bind(arg1, arg2, _)
      | Sub(arg1, arg2, _)
      | SubMov(arg1, arg2, _)
      | New(arg1, arg2, _) => {
        self.value(Some(owner), arg1);
        self.value(Some(owner), arg2);
      }
      Apply(fn_, _this, args, _) => {
        self.value(Some(owner), fn_);
        self.value(Some(owner), args);
      }
      ConstApply(fn_, this, args, _) => {
        self.value(Some(owner), fn_);
        self.value(Some(owner), this);
        self.value(Some(owner), args);
      }
      ConstSubCall(this, key, args, _) => {
        self.value(Some(owner), this);
        self.value(Some(owner), key);
        self.value(Some(owner), args);
      }
      SubCall(_this, key, args, _) | ThisSubCall(_this, key, args, _) => {
        self.value(Some(owner), key);
        self.value(Some(owner), args);
      }
      JmpIf(arg, _) | JmpIfNot(arg, _) => {
        self.value(Some(owner), arg);
      }
    };
  }

  fn body(&mut self, owner: &Pointer, body: &mut Vec<FnLine>) {
    for fn_line in body {
      match fn_line {
        FnLine::Instruction(instruction) => {
          self.instruction(owner, instruction);
        }
        FnLine::Label(..) | FnLine::Empty | FnLine::Comment(..) | FnLine::Release(..) => {}
      }
    }
  }
}
