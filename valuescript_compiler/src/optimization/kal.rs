use num_bigint::BigInt;
use valuescript_vm::{
  operations, unicode_at,
  vs_object::VsObject,
  vs_value::{number_to_index, ToVal, Val},
};

use std::{
  collections::{BTreeMap, HashMap},
  mem::take,
};

use crate::{
  asm::{self, Builtin, Function, Number, Pointer, Register, Value},
  instruction::Instruction,
  name_allocator::RegAllocator,
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
#[derive(Clone, Default)]
pub enum Kal {
  #[default]
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
    F: FnMut(&mut Kal),
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

  pub fn from_value(value: &Value) -> Self {
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

  fn try_to_value(&self) -> Option<Value> {
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
            match k.try_to_value() {
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
            let k = match k.try_to_value() {
              Some(k) => k,
              None => return None,
            };

            let v = match v.try_to_value() {
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

  // None can indicate not implemented, not just unknowable
  fn to_known_string(&self) -> Option<String> {
    match self {
      Kal::Unknown => None,
      Kal::Void => None, // 🤔
      Kal::Undefined => Some("undefined".to_string()),
      Kal::Null => Some("null".to_string()),
      Kal::Bool(b) => Some(b.to_string()),
      Kal::Number(Number(x)) => Some(x.to_string()),
      Kal::BigInt(bi) => Some(bi.to_string()),
      Kal::String(s) => Some(s.clone()),
      Kal::Array(_) => None,
      Kal::Object(_) => None,
      Kal::Register(_) => None,
      Kal::Pointer(_) => None,
      Kal::Builtin(_) => None,
    }
  }
}

#[derive(Default)]
pub struct FnState {
  pub reg_allocator: RegAllocator,
  pub pointer_kals: HashMap<Pointer, Kal>,
  pub mutable_this_established: bool,
  pub registers: BTreeMap<String, Kal>,
  pub new_instructions: Vec<Instruction>,
}

impl FnState {
  pub fn new(fn_: &Function, pointer_kals: HashMap<Pointer, Kal>) -> Self {
    let mut reg_allocator = RegAllocator::default();

    for p in &fn_.parameters {
      reg_allocator.alloc.mark_used(&p.name);
    }

    for line in &fn_.body {
      match line {
        asm::FnLine::Instruction(instr) => {
          let mut instr = instr.clone(); // TODO: Need non-mut register visitor
          instr.visit_registers_mut_rev(&mut |rvm| {
            reg_allocator.alloc.mark_used(&rvm.register.name);
          });
        }
        asm::FnLine::Label(_) => {}
        asm::FnLine::Empty => {}
        asm::FnLine::Comment(_) => {}
        asm::FnLine::Release(reg) => reg_allocator.alloc.mark_used(&reg.name),
      }
    }

    FnState {
      reg_allocator,
      pointer_kals,
      ..Default::default()
    }
  }

  pub fn clear_local(&mut self) {
    let pointer_kals = take(&mut self.pointer_kals);

    *self = Self {
      reg_allocator: take(&mut self.reg_allocator),
      pointer_kals,
      mutable_this_established: Default::default(),
      registers: Default::default(),
      new_instructions: take(&mut self.new_instructions),
    }
  }

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
    let mut new_reg: Option<String> = None;

    for kal in self.registers.values_mut() {
      if let Kal::Register(reg) = kal {
        if &reg.name == changed_reg {
          // When it's just a register, avoid using new_reg. This would just create spurious
          // renames. The real point of new_reg is to improve the *inner* knowledge of registers
          // that contain the changed_reg.
          *kal = Kal::Unknown;
          continue;
        }
      };

      kal.visit_kals_mut(&mut |sub_kal| {
        if let Kal::Register(reg) = sub_kal {
          if reg.name == *changed_reg {
            let new_reg = match &new_reg {
              Some(new_reg) => new_reg.clone(),
              None => {
                let new_reg_str = self.reg_allocator.allocate_numbered("_tmp").name;
                new_reg = Some(new_reg_str.clone());

                self.new_instructions.push(Instruction::Mov(
                  Value::Register(Register::named(changed_reg.clone())),
                  Register::named(new_reg_str.clone()),
                ));

                new_reg_str
              }
            };

            *sub_kal = Kal::Register(Register::named(new_reg));
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

      OpOptionalChain(a1, a2, dst) => {
        self.eval_arg(a1);
        self.eval_arg(a2);

        // self.apply_binary_op(a1, a2, dst, operations::op_optional_chain)
        // TODO: op_optional_chain takes mut lhs to optimize, but breaks this pattern
        self.set(dst.name.clone(), Kal::Unknown);
      }

      Sub(obj, key, dst) => {
        let obj = self.eval_arg(obj);
        let key = self.eval_arg(key);

        let item = match obj {
          Kal::String(string) => match key {
            Kal::Number(Number(i)) => match number_to_index(i) {
              Some(i) => 'b: {
                let string_bytes = string.as_bytes();

                if i >= string_bytes.len() {
                  break 'b Kal::Undefined;
                }

                match unicode_at(string_bytes, string_bytes.len(), i) {
                  Some(char) => Kal::String(char.to_string()),
                  None => Kal::String("".to_string()),
                }
              }
              None => Kal::Undefined,
            },
            Kal::String(key) => match key.as_str() {
              "length" => Kal::Number(Number(string.len() as f64)),
              _ => Kal::Unknown,
            },
            _ => Kal::Unknown,
          },
          Kal::Array(array) => match key {
            Kal::Number(Number(i)) => match number_to_index(i) {
              Some(i) => match array.values.get(i) {
                Some(item) => item.clone(),
                None => Kal::Undefined,
              },
              None => Kal::Undefined,
            },
            _ => Kal::Unknown, // TODO: Implement more cases
          },
          Kal::Object(object) => 'b: {
            let key_str = match key.to_known_string() {
              Some(s) => s,
              None => break 'b Kal::Unknown,
            };

            for (k, v) in object.properties.iter().rev() {
              match k.to_known_string() {
                Some(k) => {
                  if k == key_str {
                    break 'b v.clone();
                  }
                }
                None => break 'b Kal::Unknown,
              }
            }

            // TODO: Prototypes (currently anything with a prototype should be Kal::Unknown, but
            // when this changes Kal::Undefined could be wrong)

            Kal::Undefined
          }
          _ => Kal::Unknown, // TODO: Implement more cases
        };

        self.set(dst.name.clone(), item);
      }

      Call(a1, a2, dst) | Bind(a1, a2, dst) | SubMov(a1, a2, dst) | New(a1, a2, dst) => {
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

      JmpIf(a1, _) | JmpIfNot(a1, _) => {
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

    match instr {
      OpNot(_, dst)
      | OpBitNot(_, dst)
      | TypeOf(_, dst)
      | UnaryPlus(_, dst)
      | UnaryMinus(_, dst)
      | OpPlus(_, _, dst)
      | OpMinus(_, _, dst)
      | OpMul(_, _, dst)
      | OpDiv(_, _, dst)
      | OpMod(_, _, dst)
      | OpExp(_, _, dst)
      | OpEq(_, _, dst)
      | OpNe(_, _, dst)
      | OpTripleEq(_, _, dst)
      | OpTripleNe(_, _, dst)
      | OpAnd(_, _, dst)
      | OpOr(_, _, dst)
      | OpLess(_, _, dst)
      | OpLessEq(_, _, dst)
      | OpGreater(_, _, dst)
      | OpGreaterEq(_, _, dst)
      | OpNullishCoalesce(_, _, dst)
      | OpBitAnd(_, _, dst)
      | OpBitOr(_, _, dst)
      | OpBitXor(_, _, dst)
      | OpLeftShift(_, _, dst)
      | OpRightShift(_, _, dst)
      | OpRightShiftUnsigned(_, _, dst)
      | InstanceOf(_, _, dst)
      | In(_, _, dst)
      | Sub(_, _, dst) => {
        if let Some(value) = self.get(dst.name.clone()).try_to_value() {
          *instr = Instruction::Mov(value, dst.clone())
        }
      }

      End
      | Mov(_, _)
      | OpInc(_)
      | OpDec(_)
      | OpOptionalChain(_, _, _)
      | Call(_, _, _)
      | Apply(_, _, _, _)
      | Bind(_, _, _)
      | SubMov(_, _, _)
      | SubCall(_, _, _, _)
      | Jmp(_)
      | JmpIf(_, _)
      | JmpIfNot(_, _)
      | New(_, _, _)
      | Throw(_)
      | Import(_, _)
      | ImportStar(_, _)
      | SetCatch(_, _)
      | UnsetCatch
      | ConstSubCall(_, _, _, _)
      | RequireMutableThis
      | ThisSubCall(_, _, _, _)
      | Next(_, _)
      | UnpackIterRes(_, _, _)
      | Cat(_, _)
      | Yield(_, _)
      | YieldStar(_, _) => {}
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
      | Value::Builtin(_) => Kal::from_value(arg),
      Value::Pointer(p) => match self.pointer_kals.get(p) {
        Some(kal) => {
          if let Some(new_arg) = kal.try_to_value() {
            *arg = new_arg;
          }
          kal.clone()
        }
        None => Kal::Pointer(p.clone()),
      },
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

        match kal.try_to_value() {
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
    };
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
    // It's important that the eval happens on both args (left shouldn't emit None early) because
    // eval_arg also substitutes the Value using knowledge
    let left = self.eval_arg(left);
    let right = self.eval_arg(right);

    let left = left.try_to_val()?;
    let right = right.try_to_val()?;

    let kal = op(&left, &right).ok()?.try_to_kal()?;

    self.set(dst.name.clone(), kal);

    Some(())
  }
}
