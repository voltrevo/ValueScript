use num_bigint::BigInt;
use valuescript_vm::{
  operations,
  vs_object::VsObject,
  vs_value::{ToVal, Val},
};

use std::collections::BTreeMap;

use crate::{
  asm::{self, Builtin, Number, Pointer, Register, Value},
  instruction::Instruction,
};

use super::try_to_kal::TryToKal;

/**
 * Kal: Knowledge about a Val.
 *
 * This is used by the optimizer to make simplifications. It's a broader and more complex version
 * of Val, since every (non-external) Val can be represented as a Kal, and Kal can also represent
 * partial information about a Val, such as being a number or being equal to another register.
 *
 * This is similar to a type system. However, a type system has the constraint of needing to be
 * consistent and sensible so the programmer can use it. Kal has the advantage of only being used
 * for optimization, so we can do a lot more heuristic things like knowing when an index is within
 * the bounds of an array (without needing to nail down exactly when and why we know that in a
 * consistent way). It is also 100% mandatory that Kal is always accurate/sound (otherwise we'll
 * change program behavior due to believing false things), whereas sometimes type systems (notably
 * TypeScript) are not.
 */
#[derive(Clone)]
pub enum Kal {
  Unknown,
  Void,
  Undefined,
  Null,
  Bool(bool),
  Number(Number),
  BigInt(BigInt),
  String(String),
  Array(Box<Array>),
  Object(Box<Object>),
  Register(Register),
  Pointer(Pointer),
  Builtin(Builtin),
}

impl Default for Kal {
  fn default() -> Self {
    Kal::Unknown
  }
}

#[derive(Clone)]
pub struct Array {
  pub values: Vec<Kal>,
}

#[derive(Clone)]
pub struct Object {
  pub properties: Vec<(Kal, Kal)>,
}

impl Kal {
  fn visit_kals_mut<F>(&mut self, visit: &mut F)
  where
    F: FnMut(&mut Kal) -> (),
  {
    visit(self);

    match self {
      Kal::Array(array) => {
        for item in &mut array.values {
          item.visit_kals_mut(visit);
        }
      }
      Kal::Object(object) => {
        for (k, v) in &mut object.properties {
          k.visit_kals_mut(visit);
          v.visit_kals_mut(visit);
        }
      }
      Kal::Unknown => {}
      Kal::Void => {}
      Kal::Undefined => {}
      Kal::Null => {}
      Kal::Bool(..) => {}
      Kal::Number(..) => {}
      Kal::BigInt(..) => {}
      Kal::String(..) => {}
      Kal::Register(..) => {}
      Kal::Pointer(..) => {}
      Kal::Builtin(..) => {}
    }
  }

  fn from_value(value: &Value) -> Self {
    match value {
      Value::Void => Kal::Void,
      Value::Undefined => Kal::Undefined,
      Value::Null => Kal::Null,
      Value::Bool(bool) => Kal::Bool(*bool),
      Value::Number(Number(x)) => Kal::Number(Number(*x)),
      Value::BigInt(bi) => Kal::BigInt(bi.clone()),
      Value::String(string) => Kal::String(string.clone()),
      Value::Array(array) => Kal::Array(Box::new(Array {
        values: array.values.iter().map(Kal::from_value).collect(),
      })),
      Value::Object(object) => Kal::Object(Box::new(Object {
        properties: object
          .properties
          .iter()
          .map(|(k, v)| (Kal::from_value(k), Kal::from_value(v)))
          .collect(),
      })),
      Value::Register(reg) => Kal::Register(reg.clone()),
      Value::Pointer(p) => Kal::Pointer(p.clone()),
      Value::Builtin(b) => Kal::Builtin(b.clone()),
    }
  }

  fn to_value(&self) -> Option<Value> {
    match self {
      Kal::Unknown => None,
      Kal::Void => Some(Value::Void),
      Kal::Undefined => Some(Value::Undefined),
      Kal::Null => Some(Value::Null),
      Kal::Bool(x) => Some(Value::Bool(*x)),
      Kal::Number(Number(x)) => Some(Value::Number(Number(*x))),
      Kal::BigInt(x) => Some(Value::BigInt(x.clone())),
      Kal::String(x) => Some(Value::String(x.clone())),
      Kal::Array(x) => Some(Value::Array(Box::new(asm::Array {
        values: {
          let mut values = Vec::<asm::Value>::new();

          for k in &x.values {
            match k.to_value() {
              Some(v) => values.push(v),
              None => return None,
            }
          }

          values
        },
      }))),
      Kal::Object(x) => Some(Value::Object(Box::new(asm::Object {
        properties: {
          let mut properties = Vec::<(asm::Value, asm::Value)>::new();

          for (k, v) in &x.properties {
            let k = match k.to_value() {
              Some(k) => k,
              None => return None,
            };

            let v = match v.to_value() {
              Some(v) => v,
              None => return None,
            };

            properties.push((k, v));
          }

          properties
        },
      }))),
      Kal::Register(x) => Some(Value::Register(x.clone())),
      Kal::Pointer(x) => Some(Value::Pointer(x.clone())),
      Kal::Builtin(x) => Some(Value::Builtin(x.clone())),
    }
  }

  fn try_to_val(self) -> Option<Val> {
    Some(match self {
      Kal::Unknown => return None,
      Kal::Undefined => Val::Undefined,
      Kal::Null => Val::Null,
      Kal::Bool(b) => b.to_val(),
      Kal::Number(Number(n)) => n.to_val(),
      Kal::BigInt(n) => n.to_val(),
      Kal::String(s) => s.to_val(),
      Kal::Array(arr) => {
        let mut result = Vec::<Val>::new();

        for value in arr.values {
          result.push(value.try_to_val()?);
        }

        result.to_val()
      }
      Kal::Object(obj) => {
        let mut string_map = BTreeMap::<String, Val>::new();

        for (key, value) in obj.properties {
          string_map.insert(key.try_to_val()?.to_string(), value.try_to_val()?);
        }

        VsObject {
          string_map,
          symbol_map: Default::default(),
          prototype: None,
        }
        .to_val()
      }

      Kal::Void | Kal::Register(..) | Kal::Pointer(..) | Kal::Builtin(..) => {
        return None;
      }
    })
  }
}

#[derive(Default)]
pub struct FnState {
  pub mutable_this_established: bool,
  pub registers: BTreeMap<String, Kal>,
}

impl FnState {
  fn get_mut(&mut self, reg_name: String) -> &mut Kal {
    self.registers.entry(reg_name).or_default()
  }

  fn get(&mut self, reg_name: String) -> &Kal {
    self.get_mut(reg_name)
  }

  pub fn set(&mut self, reg_name: String, kal: Kal) {
    *self.get_mut(reg_name.clone()) = kal;
    self.handle_reg_changed(&reg_name);
  }

  fn handle_reg_changed(&mut self, changed_reg: &String) {
    for kal in self.registers.values_mut() {
      kal.visit_kals_mut(&mut |sub_kal| {
        if let Kal::Register(reg) = sub_kal {
          if reg.name == *changed_reg {
            *sub_kal = Kal::Unknown;
          }
        }
      });
    }
  }

  pub fn eval_instruction(&mut self, instr: &mut Instruction) {
    use Instruction::*;

    match instr {
      End => {}
      Mov(arg, dst) => {
        let arg = self.eval_arg(arg);
        self.set(dst.name.clone(), arg);
      }

      OpInc(reg) => {
        // TODO: Use apply_binary_op?

        let new_value = match self.get(reg.name.clone()) {
          Kal::Number(Number(x)) => Kal::Number(Number(x + 1.0)),
          Kal::BigInt(x) => Kal::BigInt(x + BigInt::from(1)),
          _ => Kal::Unknown,
        };

        self.set(reg.name.clone(), new_value);
      }
      OpDec(reg) => {
        // TODO: Use apply_binary_op?

        let new_value = match self.get(reg.name.clone()) {
          Kal::Number(Number(x)) => Kal::Number(Number(x - 1.0)),
          Kal::BigInt(x) => Kal::BigInt(x - BigInt::from(1)),
          _ => Kal::Unknown,
        };

        self.set(reg.name.clone(), new_value);
      }

      OpNot(a1, dst) => self.apply_unary_op(a1, dst, operations::op_not),
      OpBitNot(a1, dst) => self.apply_unary_op(a1, dst, operations::op_bit_not),
      TypeOf(a1, dst) => self.apply_unary_op(a1, dst, operations::op_typeof),
      UnaryPlus(a1, dst) => self.apply_unary_op(a1, dst, operations::op_unary_plus),
      UnaryMinus(a1, dst) => self.apply_unary_op(a1, dst, operations::op_unary_minus),
      Import(a1, dst) | ImportStar(a1, dst) | Cat(a1, dst) => {
        self.eval_arg(a1);

        // TODO: cat
        self.set(dst.name.clone(), Kal::Unknown);
      }

      Yield(a1, dst) | YieldStar(a1, dst) => {
        self.eval_arg(a1);
        self.set(dst.name.clone(), Kal::Unknown);
      }

      Throw(a1) => {
        self.eval_arg(a1);
      }

      OpPlus(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_plus),
      OpMinus(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_minus),
      OpMul(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_mul),
      OpDiv(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_div),
      OpMod(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_mod),
      OpExp(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_exp),
      OpEq(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_eq),
      OpNe(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_ne),
      OpTripleEq(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_triple_eq),
      OpTripleNe(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_triple_ne),
      OpAnd(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_and),
      OpOr(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_or),
      OpLess(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_less),
      OpLessEq(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_less_eq),
      OpGreater(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_greater),
      OpGreaterEq(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_greater_eq),
      OpNullishCoalesce(a1, a2, dst) => {
        self.apply_binary_op(a1, a2, dst, operations::op_nullish_coalesce)
      }
      OpOptionalChain(a1, a2, dst) => {
        self.eval_arg(a1);
        self.eval_arg(a2);

        // self.apply_binary_op(a1, a2, dst, operations::op_optional_chain)
        // TODO: op_optional_chain takes mut lhs to optimize, but breaks this pattern
        self.set(dst.name.clone(), Kal::Unknown);
      }
      OpBitAnd(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_bit_and),
      OpBitOr(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_bit_or),
      OpBitXor(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_bit_xor),
      OpLeftShift(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_left_shift),
      OpRightShift(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_right_shift),
      OpRightShiftUnsigned(a1, a2, dst) => {
        self.apply_binary_op(a1, a2, dst, operations::op_right_shift_unsigned)
      }
      InstanceOf(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_instance_of),
      In(a1, a2, dst) => self.apply_binary_op(a1, a2, dst, operations::op_in),

      Call(a1, a2, dst)
      | Bind(a1, a2, dst)
      | Sub(a1, a2, dst)
      | SubMov(a1, a2, dst)
      | New(a1, a2, dst) => {
        self.eval_arg(a1);
        self.eval_arg(a2);
        self.set(dst.name.clone(), Kal::Unknown);
      }

      Apply(a, this, a3, dst) | SubCall(this, a, a3, dst) | ThisSubCall(this, a, a3, dst) => {
        // TODO: Consider ordering here (only .take consideration (?))
        self.eval_arg(a);
        self.eval_arg(a3);

        self.set(this.name.clone(), Kal::Unknown);
        self.set(dst.name.clone(), Kal::Unknown);
      }

      ConstSubCall(a1, a2, a3, dst) => {
        self.eval_arg(a1);
        self.eval_arg(a2);
        self.eval_arg(a3);

        self.set(dst.name.clone(), Kal::Unknown);
      }

      JmpIf(a1, _) => {
        self.eval_arg(a1);
      }

      Jmp(_) => {}
      SetCatch(_, _) => {}
      UnsetCatch => {}
      RequireMutableThis => {
        self.mutable_this_established = true;
      }
      Next(iter, dst) => {
        self.set(iter.name.clone(), Kal::Unknown);
        self.set(dst.name.clone(), Kal::Unknown);
      }
      UnpackIterRes(iter_res, value_reg, done) => {
        self.set(iter_res.name.clone(), Kal::Void);
        self.set(value_reg.name.clone(), Kal::Unknown);
        self.set(done.name.clone(), Kal::Unknown);
      }
    }
  }

  fn eval_arg(&mut self, arg: &mut Value) -> Kal {
    match arg {
      Value::Void
      | Value::Undefined
      | Value::Null
      | Value::Bool(_)
      | Value::Number(_)
      | Value::BigInt(_)
      | Value::String(_)
      | Value::Pointer(_)
      | Value::Builtin(_) => Kal::from_value(arg),
      Value::Array(array) => {
        let mut values = Vec::<Kal>::new();

        for item in &mut array.values {
          values.push(self.eval_arg(item));
        }

        Kal::Array(Box::new(Array { values }))
      }
      Value::Object(object) => {
        let mut properties = Vec::<(Kal, Kal)>::new();

        for (k, v) in &mut object.properties {
          let k = self.eval_arg(k);
          let v = self.eval_arg(v);

          properties.push((k, v));
        }

        Kal::Object(Box::new(Object { properties }))
      }
      Value::Register(reg) => {
        let kal = self.get(reg.name.clone()).clone();

        let is_take = reg.take;

        if is_take {
          self.set(reg.name.clone(), Kal::Void);
        }

        match kal.to_value() {
          Some(v) => {
            // Note: if `reg.take` was true, then we're removing that take operation from the
            // register here. This should be ok because well-formed programs should never read from
            // a taken register, but we might need to revise this in the future. It definitely means
            // it's possible for the optimizer to break hand-written assembly.
            *arg = v;

            kal
          }
          None => match is_take {
            true => Kal::Unknown,
            false => Kal::Register(reg.clone()),
          },
        }
      }
    }
  }

  fn apply_unary_op(&mut self, arg: &mut Value, dst: &Register, op: fn(input: &Val) -> Val) {
    match self.apply_unary_op_impl(arg, dst, op) {
      Some(_) => {}
      None => {
        self.set(dst.name.clone(), Kal::Unknown);
      }
    }
  }

  fn apply_unary_op_impl(
    &mut self,
    arg: &mut Value,
    dst: &Register,
    op: fn(input: &Val) -> Val,
  ) -> Option<()> {
    let arg = self.eval_arg(arg).try_to_val()?;
    let kal = op(&arg).try_to_kal()?;

    self.set(dst.name.clone(), kal);

    Some(())
  }

  fn apply_binary_op(
    &mut self,
    left: &mut Value,
    right: &mut Value,
    dst: &Register,
    op: fn(left: &Val, right: &Val) -> Result<Val, Val>,
  ) {
    match self.apply_binary_op_impl(left, right, dst, op) {
      Some(_) => {}
      None => {
        self.set(dst.name.clone(), Kal::Unknown);
      }
    }
  }

  fn apply_binary_op_impl(
    &mut self,
    left: &mut Value,
    right: &mut Value,
    dst: &Register,
    op: fn(left: &Val, right: &Val) -> Result<Val, Val>,
  ) -> Option<()> {
    let left = self.eval_arg(left).try_to_val()?;
    let right = self.eval_arg(right).try_to_val()?;
    let kal = op(&left, &right).ok()?.try_to_kal()?;

    self.set(dst.name.clone(), kal);

    Some(())
  }
}
