use std::cell::RefCell;
use std::mem::take;

use swc_common::Spanned;

use crate::asm::{Array, Instruction, Label, Number, Object, Register, Structured, Value};
use crate::diagnostic::{Diagnostic, DiagnosticContainer, DiagnosticReporter};
use crate::function_compiler::{FunctionCompiler, Functionish};
use crate::ident::Ident as CrateIdent;
use crate::scope::{NameId, OwnerId};
use crate::scope_analysis::{fn_to_owner_id, NameType};
use crate::target_accessor::TargetAccessor;

#[derive(Debug, Default)]
pub struct CompiledExpression {
  pub value: Value,

  pub nested_registers: Vec<Register>,
  pub release_checker: ReleaseChecker,
}

#[derive(Debug, Default)]
pub struct ReleaseChecker {
  pub has_unreleased_registers: bool,
}

impl ReleaseChecker {
  pub fn new(has_unreleased_registers: bool) -> ReleaseChecker {
    ReleaseChecker {
      has_unreleased_registers,
    }
  }
}

impl CompiledExpression {
  pub fn empty() -> CompiledExpression {
    CompiledExpression {
      value: Value::Void,
      nested_registers: vec![],
      release_checker: ReleaseChecker::new(false),
    }
  }

  pub fn new(value: Value, nested_registers: Vec<Register>) -> CompiledExpression {
    let has_unreleased_registers = !nested_registers.is_empty();

    CompiledExpression {
      value,
      nested_registers,
      release_checker: ReleaseChecker::new(has_unreleased_registers),
    }
  }
}

impl Drop for ReleaseChecker {
  fn drop(&mut self) {
    assert!(
      !self.has_unreleased_registers,
      "CompiledExpression dropped with unreleased registers"
    );
  }
}

pub struct ExpressionCompiler<'a, 'fnc> {
  pub fnc: &'a mut FunctionCompiler<'fnc>,
}

impl<'a, 'fnc> DiagnosticContainer for ExpressionCompiler<'a, 'fnc> {
  fn diagnostics_mut(&self) -> &RefCell<Vec<Diagnostic>> {
    self.fnc.diagnostics_mut()
  }
}

impl<'a, 'fnc> ExpressionCompiler<'a, 'fnc> {
  pub fn compile_top_level(
    &mut self,
    expr: &swc_ecma_ast::Expr,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    use swc_ecma_ast::Expr::*;

    match expr {
      Assign(assign_exp) => self.assign_expression(assign_exp, true, target_register),
      _ => self.compile(expr, target_register),
    }
  }

  pub fn compile(
    &mut self,
    expr: &swc_ecma_ast::Expr,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    use swc_ecma_ast::Expr::*;

    match expr {
      This(this) => self.ident(&CrateIdent::this(this.span), target_register),
      Array(array_exp) => self.array_expression(array_exp, target_register),
      Object(object_exp) => self.object_expression(object_exp, target_register),
      Fn(fn_) => self.fn_expression(fn_, target_register),
      Unary(un_exp) => self.unary_expression(un_exp, target_register),
      Update(update_exp) => self.update_expression(update_exp, target_register),
      Bin(bin_exp) => self.binary_expression(bin_exp, target_register),
      Assign(assign_exp) => self.assign_expression(assign_exp, false, target_register),
      Member(member_exp) => self.member_expression(member_exp, target_register),
      SuperProp(super_prop) => {
        self.todo(super_prop.span, "SuperProp expression");
        CompiledExpression::empty()
      }
      Cond(cond_exp) => self.cond_expression(cond_exp, target_register),
      Call(call_exp) => match &call_exp.callee {
        swc_ecma_ast::Callee::Expr(callee_expr) => match &**callee_expr {
          swc_ecma_ast::Expr::Member(member_expr) => {
            self.method_call_expression(member_expr, &call_exp.args, target_register)
          }
          _ => self.call_expression(call_exp, target_register),
        },
        _ => {
          self.todo(call_exp.callee.span(), "non-expression callee");

          CompiledExpression::empty()
        }
      },
      New(new_exp) => self.new_expression(new_exp, target_register),
      Seq(seq_exp) => {
        for i in 0..(seq_exp.exprs.len() - 1) {
          self.compile_into(&seq_exp.exprs[i], Register::ignore());
        }

        self.compile(seq_exp.exprs.last().unwrap(), target_register)
      }
      Ident(ident) => self.ident(&CrateIdent::from_swc_ident(ident), target_register),
      Lit(lit) => self.compile_literal(lit).to_ce(),
      Tpl(tpl) => self.template_literal(tpl, target_register),
      TaggedTpl(tagged_tpl) => {
        self.todo(tagged_tpl.span, "TaggedTpl expression");
        CompiledExpression::empty()
      }
      Arrow(arrow) => self.arrow_expression(arrow, target_register),
      Class(class_exp) => {
        // TODO: Handle captures
        let p = self
          .fnc
          .mc
          .compile_class(None, class_exp.ident.as_ref(), &class_exp.class);

        CompiledExpression::new(Value::Pointer(p), vec![])
      }
      Yield(yield_expr) => self.yield_expr(yield_expr, target_register),
      MetaProp(meta_prop) => {
        self.todo(meta_prop.span, "MetaProp expression");
        CompiledExpression::empty()
      }
      Await(await_exp) => {
        self.todo(await_exp.span, "Await expression");
        CompiledExpression::empty()
      }
      Paren(p) => self.compile(&p.expr, target_register),
      JSXMember(jsx_member) => {
        self.todo(jsx_member.span(), "JSXMember expression");
        CompiledExpression::empty()
      }
      JSXNamespacedName(jsx_namespaced_name) => {
        self.todo(jsx_namespaced_name.span(), "JSXNamespacedName expression");
        CompiledExpression::empty()
      }
      JSXEmpty(jsx_empty) => {
        self.todo(jsx_empty.span(), "JSXEmpty expression");
        CompiledExpression::empty()
      }
      JSXElement(jsx_element) => self.jsx_element(jsx_element, target_register),
      JSXFragment(jsx_fragment) => self.jsx_fragment(jsx_fragment, target_register),
      TsTypeAssertion(ts_type_assertion) => self.compile(&ts_type_assertion.expr, target_register),
      TsConstAssertion(ts_const_assertion) => {
        self.compile(&ts_const_assertion.expr, target_register)
      }
      TsNonNull(ts_non_null_exp) => self.compile(&ts_non_null_exp.expr, target_register),
      TsAs(ts_as_exp) => self.compile(&ts_as_exp.expr, target_register),
      TsInstantiation(ts_instantiation) => {
        self.todo(ts_instantiation.span, "TsInstantiation expression");

        CompiledExpression::empty()
      }
      PrivateName(private_name) => {
        self.todo(private_name.span, "PrivateName expression");
        CompiledExpression::empty()
      }
      OptChain(opt_chain) => {
        self.todo(opt_chain.span, "OptChain expression");
        CompiledExpression::empty()
      }
      Invalid(invalid) => {
        self.error(invalid.span, "Invalid expression");

        CompiledExpression::empty()
      }
    }
  }

  pub fn compile_into(&mut self, expr: &swc_ecma_ast::Expr, target_register: Register) {
    let ce = self.compile(expr, Some(target_register.clone()));
    let mut in_target = false;

    if let Value::Register(ce_reg) = &ce.value {
      if ce_reg == &target_register {
        in_target = true;
        // Result is already in the target register, no mov needed
      }
    }

    if !in_target && !target_register.is_ignore() {
      // Put the value into the target
      self
        .fnc
        .push(Instruction::Mov(ce.value.clone(), target_register));
    }

    self.fnc.release_ce(ce);
  }

  pub fn unary_expression(
    &mut self,
    un_exp: &swc_ecma_ast::UnaryExpr,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    match UnaryOpType::from_unary_op(un_exp.op) {
      UnaryOpType::Ordinary(ordinary_op) => {
        self.ordinary_unary_expression(un_exp, ordinary_op, target_register)
      }
      UnaryOpType::Void => {
        let arg = self.compile(&un_exp.arg, None);
        self.fnc.release_ce(arg);

        CompiledExpression::new(Value::Undefined, vec![])
      }
      UnaryOpType::Delete => {
        let member_expr = match &*un_exp.arg {
          swc_ecma_ast::Expr::Member(e) => e,
          _ => {
            self.error(
              un_exp.arg.span(),
              "Cannot apply delete operator to non-member expression (ie bad: `delete x`, good: \
                `delete x.y`)",
            );

            return CompiledExpression::new(
              Value::Register(self.fnc.allocate_numbered_reg("_bad_delete")),
              vec![],
            );
          }
        };

        let mut ta = TargetAccessor::compile(self, &member_expr.obj, true);
        let obj_target = ta.read(self);

        let prop = self.member_prop(&member_expr.prop, None);
        let mut nested_registers = Vec::<Register>::new();

        let dst = match target_register {
          Some(t) => t,
          None => {
            let dst = self.fnc.allocate_tmp();
            nested_registers.push(dst.clone());
            dst
          }
        };

        self.fnc.push(Instruction::Delete(
          obj_target.clone(),
          prop.value.clone(),
          dst.clone(),
        ));

        self.fnc.release_ce(prop);

        ta.packup(self, false);

        CompiledExpression::new(Value::Register(dst), nested_registers)
      }
    }
  }

  pub fn ordinary_unary_expression(
    &mut self,
    un_exp: &swc_ecma_ast::UnaryExpr,
    op: OrdinaryUnaryOp,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<Register>::new();

    let arg = self.compile(&un_exp.arg, None);

    let target: Register = match &target_register {
      None => {
        let res = self.fnc.allocate_tmp();
        nested_registers.push(res.clone());
        res
      }
      Some(t) => t.clone(),
    };

    self
      .fnc
      .push(op.make_instruction(arg.value.clone(), target.clone()));

    self.fnc.release_ce(arg);

    CompiledExpression::new(Value::Register(target), nested_registers)
  }

  pub fn binary_expression(
    &mut self,
    bin: &swc_ecma_ast::BinExpr,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<Register>::new();

    let left = self.compile(&bin.left, None);
    let right = self.compile(&bin.right, None);

    // FIXME: && and || need to avoid executing the right side where applicable
    // (mandatory if they mutate)

    let target: Register = match &target_register {
      None => {
        let res = self.fnc.allocate_tmp();
        nested_registers.push(res.clone());
        res
      }
      Some(t) => t.clone(),
    };

    self.fnc.push(make_binary_op(
      bin.op,
      left.value.clone(),
      right.value.clone(),
      target.clone(),
    ));

    self.fnc.release_ce(left);
    self.fnc.release_ce(right);

    CompiledExpression::new(Value::Register(target), nested_registers)
  }

  pub fn get_register_for_ident_mutation(&mut self, ident: &CrateIdent) -> Register {
    let (reg, err_msg) = match self.fnc.lookup_value(ident) {
      Some(Value::Register(reg)) => (Some(reg), None),
      lookup_result => (
        None,
        Some(format!(
          "Invalid: Can't mutate {} because its lookup result is {:?}",
          ident.sym, lookup_result,
        )),
      ),
    };

    if let Some(err_msg) = err_msg {
      self.error(ident.span, &err_msg);
    }

    if let Some(reg) = reg {
      return reg;
    }

    self
      .fnc
      .allocate_numbered_reg(&format!("_couldnt_mutate_{}_", ident.sym))
  }

  pub fn assign_expression(
    &mut self,
    assign_expr: &swc_ecma_ast::AssignExpr,
    is_top_level: bool,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    match get_binary_op_for_assign_op(assign_expr.op) {
      None => self.assign_expr_eq(assign_expr, is_top_level, target_register),
      Some(binary_op) => {
        self.assign_expr_compound(assign_expr, is_top_level, binary_op, target_register)
      }
    }
  }

  pub fn assign_expr_eq(
    &mut self,
    assign_expr: &swc_ecma_ast::AssignExpr,
    is_top_level: bool,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let mut at = match &assign_expr.left {
      swc_ecma_ast::PatOrExpr::Pat(pat) => match &**pat {
        swc_ecma_ast::Pat::Ident(ident) => {
          TargetAccessor::compile_ident(self, &CrateIdent::from_swc_ident(&ident.id))
        }
        swc_ecma_ast::Pat::Expr(expr) => TargetAccessor::compile(self, expr, true),
        _ => return self.assign_pat_eq(pat, &assign_expr.right, target_register),
      },
      swc_ecma_ast::PatOrExpr::Expr(expr) => TargetAccessor::compile(self, expr, true),
    };

    let rhs = match is_top_level {
      true => match at.direct_register() {
        Some(at) => {
          self.compile_into(&assign_expr.right, at.clone());
          Value::Register(at).to_ce()
        }
        None => self.compile(&assign_expr.right, None),
      },
      false => self.compile(&assign_expr.right, None),
    };

    let at_is_register = matches!(&at, TargetAccessor::Register(_));

    if is_top_level && at_is_register {
      // at is already assigned by compiling directly into at.direct_register()
      // and also doesn't need to be packed up, since it's just a register
    } else {
      at.assign_and_packup(self, &rhs.value, false);
    }

    rhs
  }

  pub fn assign_pat_eq(
    &mut self,
    pat: &swc_ecma_ast::Pat,
    assign_expr_right: &swc_ecma_ast::Expr,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let mut nested_registers = vec![];

    let rhs_reg = match target_register {
      Some(reg) => reg,
      None => {
        let reg = self.fnc.allocate_tmp();
        nested_registers.push(reg.clone());
        reg
      }
    };

    self.compile_into(assign_expr_right, rhs_reg.clone());

    self.pat(pat, &rhs_reg, true);

    CompiledExpression::new(Value::Register(rhs_reg), nested_registers)
  }

  pub fn assign_expr_compound(
    &mut self,
    assign_expr: &swc_ecma_ast::AssignExpr,
    is_top_level: bool,
    binary_op: swc_ecma_ast::BinaryOp,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    use swc_ecma_ast::Pat;
    use swc_ecma_ast::PatOrExpr;

    let mut target = match &assign_expr.left {
      PatOrExpr::Expr(expr) => TargetAccessor::compile(self, expr, true),
      PatOrExpr::Pat(pat) => match &**pat {
        Pat::Ident(ident) => TargetAccessor::Register(
          self.get_register_for_ident_mutation(&CrateIdent::from_swc_ident(&ident.id)),
        ),
        _ => {
          self.error(pat.span(), "Invalid lvalue expression");
          let bad_reg = self.fnc.allocate_numbered_reg("_bad_lvalue");

          TargetAccessor::Register(bad_reg)
        }
      },
    };

    let mut nested_registers = vec![];

    let rhs_reg = match &target_register {
      Some(reg) => reg.clone(),
      None => {
        let reg = self.fnc.allocate_tmp();
        nested_registers.push(reg.clone());
        reg
      }
    };

    self.compile_into(&assign_expr.right, rhs_reg.clone());

    // TODO: Does target_read need to be released?
    let target_read = target.read(self);

    self.fnc.push(make_binary_op(
      binary_op,
      Value::Register(target_read.clone()),
      Value::Register(rhs_reg.clone()),
      target_read.clone(),
    ));

    if !is_top_level {
      // Technically we should do this unconditionally. When is_top_level is true, we're lying about
      // rhs_reg being the result, but the idea is that we don't use the result when is_top_level is
      // true. There's probably a better way to handle this.
      self.fnc.push(Instruction::Mov(
        Value::Register(target_read.clone()),
        rhs_reg.clone(),
      ));
    }

    target.assign_and_packup(self, &Value::Register(target_read), false);

    CompiledExpression::new(Value::Register(rhs_reg), nested_registers)
  }

  pub fn array_expression(
    &mut self,
    array_exp: &swc_ecma_ast::ArrayLit,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let args = array_exp.elems.iter().map(|x| x.as_ref());

    self.args(args, target_register)
  }

  pub fn object_expression(
    &mut self,
    object_exp: &swc_ecma_ast::ObjectLit,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let mut object_asm = Object::default();

    let mut sub_nested_registers = Vec::<Register>::new();

    for i in 0..object_exp.props.len() {
      use swc_ecma_ast::Prop;
      use swc_ecma_ast::PropOrSpread;

      match &object_exp.props[i] {
        PropOrSpread::Spread(spread) => {
          self.todo(spread.span(), "spread expression");
        }
        PropOrSpread::Prop(prop) => match &**prop {
          Prop::Shorthand(ident) => {
            let prop_key = Value::String(ident.sym.to_string());

            let mut compiled_value = self.ident(&CrateIdent::from_swc_ident(ident), None);
            sub_nested_registers.append(&mut compiled_value.nested_registers);
            compiled_value.release_checker.has_unreleased_registers = false;
            let prop_value = compiled_value.value;

            object_asm.properties.push((prop_key, prop_value));
          }
          Prop::KeyValue(kv) => {
            let mut compiled_key = self.prop_name(&kv.key);
            compiled_key.release_checker.has_unreleased_registers = false;
            sub_nested_registers.append(&mut compiled_key.nested_registers);

            let prop_key = compiled_key.value;

            let mut compiled_value = self.compile(&kv.value, None);
            compiled_value.release_checker.has_unreleased_registers = false;
            sub_nested_registers.append(&mut compiled_value.nested_registers);
            let prop_value = compiled_value.value;

            object_asm.properties.push((prop_key, prop_value));
          }
          Prop::Assign(assign) => self.todo(assign.span(), "Assign prop"),
          Prop::Getter(getter) => self.todo(getter.span(), "Getter prop"),
          Prop::Setter(setter) => self.todo(setter.span(), "Setter prop"),
          Prop::Method(method) => {
            let mut compiled_key = self.prop_name(&method.key);
            compiled_key.release_checker.has_unreleased_registers = false;
            sub_nested_registers.append(&mut compiled_key.nested_registers);

            let prop_key = compiled_key.value;

            let fn_ident = match &method.key {
              swc_ecma_ast::PropName::Ident(ident) => Some(ident.clone()),
              _ => None,
            };

            let fn_name = fn_ident.clone().map(|ident| ident.sym.to_string());

            let p = match &fn_name {
              Some(name) => self.fnc.allocate_defn(name),
              None => self.fnc.allocate_defn_numbered("_anon"),
            };

            FunctionCompiler::new(self.fnc.mc).compile(
              p.clone(),
              Functionish::Fn(fn_ident, method.function.clone()),
            );

            object_asm.properties.push((prop_key, Value::Pointer(p)));
          }
        },
      }
    }

    match target_register {
      None => CompiledExpression::new(Value::Object(Box::new(object_asm)), sub_nested_registers),
      Some(tr) => {
        self.fnc.push(Instruction::Mov(
          Value::Object(Box::new(object_asm)),
          tr.clone(),
        ));

        for reg in sub_nested_registers {
          self.fnc.release_reg(&reg);
        }

        CompiledExpression::new(Value::Register(tr), vec![])
      }
    }
  }

  pub fn prop_name(&mut self, prop_name: &swc_ecma_ast::PropName) -> CompiledExpression {
    use swc_ecma_ast::PropName;

    let mut nested_registers = Vec::<Register>::new();

    let value = match &prop_name {
      PropName::Ident(ident) => Value::String(ident.sym.to_string()),
      PropName::Str(str_) => Value::String(str_.value.to_string()),
      PropName::Num(num) =>
      // TODO: JS number stringification (different from rust)
      {
        Value::String(num.value.to_string()) // TODO: Can we just use Value::Number here?
      }
      PropName::Computed(comp) => {
        // TODO: Always using a register is maybe not ideal
        // At the least, the assembly supports definitions and should
        // maybe support any value here
        let reg = self.fnc.allocate_numbered_reg("_computed_key");
        self.compile_into(&comp.expr, reg.clone());
        nested_registers.push(reg.clone());

        Value::Register(reg)
      }
      PropName::BigInt(bigint) => Value::String(bigint.value.to_string()),
    };

    CompiledExpression::new(value, nested_registers)
  }

  pub fn member_prop(
    &mut self,
    member_prop: &swc_ecma_ast::MemberProp,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    match member_prop {
      swc_ecma_ast::MemberProp::Ident(ident) => Value::String(ident.sym.to_string()).to_ce(),
      swc_ecma_ast::MemberProp::Computed(computed) => self.compile(&computed.expr, target_register),
      swc_ecma_ast::MemberProp::PrivateName(private_name) => {
        self.todo(private_name.span(), "private name member property");

        CompiledExpression::empty()
      }
    }
  }

  pub fn member_expression(
    &mut self,
    member_exp: &swc_ecma_ast::MemberExpr,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let compiled_obj = self.compile(&member_exp.obj, None);
    let compiled_prop = self.member_prop(&member_exp.prop, None);

    let tmp_dest: Register;

    let (dest, nested_registers) = match &target_register {
      Some(tr) => (tr, vec![]),
      None => {
        tmp_dest = self.fnc.allocate_tmp();
        (&tmp_dest, vec![tmp_dest.clone()])
      }
    };

    self.fnc.push(Instruction::Sub(
      compiled_obj.value.clone(),
      compiled_prop.value.clone(),
      dest.clone(),
    ));

    self.fnc.release_ce(compiled_obj);
    self.fnc.release_ce(compiled_prop);

    CompiledExpression::new(Value::Register(dest.clone()), nested_registers)
  }

  pub fn cond_expression(
    &mut self,
    cond_exp: &swc_ecma_ast::CondExpr,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let mut nested_registers = vec![];

    let dst = match target_register {
      Some(reg) => reg,
      None => {
        let tmp = self.fnc.allocate_tmp();
        nested_registers.push(tmp.clone());

        tmp
      }
    };

    self.compile_into(&cond_exp.test, dst.clone());

    let true_label = Label {
      name: self.fnc.label_allocator.allocate_numbered("cond_true"),
    };

    let cond_end_label = Label {
      name: self.fnc.label_allocator.allocate_numbered("cond_end"),
    };

    self.fnc.push(Instruction::JmpIf(
      Value::Register(dst.clone()),
      true_label.ref_(),
    ));

    self.compile_into(&cond_exp.alt, dst.clone());
    self.fnc.push(Instruction::Jmp(cond_end_label.ref_()));

    self.fnc.label(true_label);

    self.compile_into(&cond_exp.cons, dst.clone());

    self.fnc.label(cond_end_label);

    CompiledExpression::new(Value::Register(dst), nested_registers)
  }

  pub fn update_expression(
    &mut self,
    update_exp: &swc_ecma_ast::UpdateExpr,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let mut target = TargetAccessor::compile(self, &update_exp.arg, true);
    let target_read = target.read(self);

    let res = match update_exp.prefix {
      true => {
        self
          .fnc
          .push(make_update_op(update_exp.op, target_read.clone()));

        let mut nested_registers = Vec::<Register>::new();

        let result_reg = match &target {
          TargetAccessor::Register(reg) => {
            let res = match &target_register {
              Some(tr) => tr.clone(),
              None => {
                let res = self.fnc.allocate_tmp();
                nested_registers.push(res.clone());

                res
              }
            };

            self
              .fnc
              .push(Instruction::Mov(Value::Register(reg.clone()), res.clone()));

            res
          }
          TargetAccessor::Nested(nta) => match target_register {
            Some(tr) => {
              self.fnc.push(Instruction::Mov(
                Value::Register(nta.register.clone()),
                tr.clone(),
              ));

              tr
            }
            None => {
              let res = self.fnc.allocate_tmp();
              nested_registers.push(res.clone());

              self.fnc.push(Instruction::Mov(
                Value::Register(nta.register.clone()),
                res.clone(),
              ));

              res
            }
          },
        };

        CompiledExpression::new(Value::Register(result_reg), nested_registers)
      }
      false => {
        let mut nested_registers = Vec::<Register>::new();

        let old_value_reg = match target_register {
          Some(tr) => tr,
          None => {
            let res = self.fnc.allocate_tmp();
            nested_registers.push(res.clone());

            res
          }
        };

        self.fnc.push(Instruction::Mov(
          Value::Register(target_read.clone()),
          old_value_reg.clone(),
        ));

        self
          .fnc
          .push(make_update_op(update_exp.op, target_read.clone()));

        CompiledExpression::new(Value::Register(old_value_reg), nested_registers)
      }
    };

    target.assign_and_packup(self, &Value::Register(target_read), false);

    res
  }

  pub fn call_expression(
    &mut self,
    call_exp: &swc_ecma_ast::CallExpr,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<Register>::new();

    let callee = match &call_exp.callee {
      swc_ecma_ast::Callee::Expr(expr) => self.compile(expr, None),
      _ => {
        self.todo(call_exp.callee.span(), "non-expression callee");

        CompiledExpression::empty()
      }
    };

    let args = call_exp.args.iter().map(Some);
    let compiled_args = self.args(args, None);

    let dest = match target_register {
      Some(tr) => tr,
      None => {
        let tmp = self.fnc.allocate_tmp();
        nested_registers.push(tmp.clone());

        tmp
      }
    };

    self.fnc.push(Instruction::Call(
      callee.value.clone(),
      compiled_args.value.clone(),
      dest.clone(),
    ));

    self.fnc.release_ce(compiled_args);
    self.fnc.release_ce(callee);

    CompiledExpression::new(Value::Register(dest), nested_registers)
  }

  pub fn new_expression(
    &mut self,
    new_exp: &swc_ecma_ast::NewExpr,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    // TODO: Try to deduplicate with call_expression

    let mut nested_registers = Vec::<Register>::new();

    let callee = self.compile(&new_exp.callee, None);

    let compiled_args = match &new_exp.args {
      None => CompiledExpression::new(Value::Array(Box::default()), vec![]),
      Some(new_exp_args) => {
        let args = new_exp_args.iter().map(Some);
        self.args(args, None)
      }
    };

    let dest = match target_register {
      Some(tr) => tr,
      None => {
        let tmp = self.fnc.allocate_tmp();
        nested_registers.push(tmp.clone());

        tmp
      }
    };

    self.fnc.push(Instruction::New(
      callee.value.clone(),
      compiled_args.value.clone(),
      dest.clone(),
    ));

    self.fnc.release_ce(callee);
    self.fnc.release_ce(compiled_args);

    CompiledExpression::new(Value::Register(dest), nested_registers)
  }

  pub fn method_call_expression(
    &mut self,
    callee_expr: &swc_ecma_ast::MemberExpr,
    args: &[swc_ecma_ast::ExprOrSpread],
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<Register>::new();
    let mut sub_nested_registers = Vec::<Register>::new();

    enum TargetAccessorOrCompiledExpression {
      TargetAccessor(TargetAccessor),
      CompiledExpression(CompiledExpression),
    }

    let obj = match TargetAccessor::is_eligible_expr(self, &callee_expr.obj) {
      true => TargetAccessorOrCompiledExpression::TargetAccessor(TargetAccessor::compile(
        self,
        &callee_expr.obj,
        true,
      )),
      false => {
        TargetAccessorOrCompiledExpression::CompiledExpression(self.compile(&callee_expr.obj, None))
      }
    };

    let obj_value = match &obj {
      TargetAccessorOrCompiledExpression::TargetAccessor(ta) => Value::Register(ta.read(self)),
      TargetAccessorOrCompiledExpression::CompiledExpression(ce) => ce.value.clone(),
    };

    let mut prop = self.member_prop(&callee_expr.prop, None);

    prop.release_checker.has_unreleased_registers = false;
    sub_nested_registers.append(&mut prop.nested_registers);

    let compiled_args = {
      let args_iter = args.iter().map(Some);
      self.args(args_iter, None)
    };

    let dest = match target_register {
      Some(tr) => tr,
      None => {
        let tmp = self.fnc.allocate_tmp();
        nested_registers.push(tmp.clone());

        tmp
      }
    };

    match obj {
      TargetAccessorOrCompiledExpression::TargetAccessor(mut ta) => {
        let targets_this = ta.targets_this();

        let obj_reg = match obj_value.clone() {
          Value::Register(obj_reg) => obj_reg,
          _ => panic!("Read of target accessor should always be a register"),
        };

        self.fnc.push(match targets_this {
          false => Instruction::SubCall(
            obj_reg,
            prop.value,
            compiled_args.value.clone(),
            dest.clone(),
          ),
          true => Instruction::ThisSubCall(
            obj_reg,
            prop.value,
            compiled_args.value.clone(),
            dest.clone(),
          ),
        });

        self.fnc.release_ce(compiled_args);

        ta.assign_and_packup(self, &obj_value, targets_this);
      }
      TargetAccessorOrCompiledExpression::CompiledExpression(ce) => {
        self.fnc.push(Instruction::ConstSubCall(
          obj_value.clone(),
          prop.value,
          compiled_args.value.clone(),
          dest.clone(),
        ));

        self.fnc.release_ce(compiled_args);
        self.fnc.release_ce(ce);
      }
    }

    for reg in sub_nested_registers {
      self.fnc.release_reg(&reg);
    }

    CompiledExpression::new(Value::Register(dest), nested_registers)
  }

  pub fn fn_expression(
    &mut self,
    fn_: &swc_ecma_ast::FnExpr,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let fn_name = fn_.ident.clone().map(|ident| ident.sym.to_string());

    let definition_pointer = match &fn_name {
      Some(name) => self.fnc.allocate_defn(name),
      None => self.fnc.allocate_defn_numbered("_anon"),
    };

    let capture_params = self
      .fnc
      .mc
      .scope_analysis
      .get_register_captures(&fn_to_owner_id(fn_.ident.as_ref(), &fn_.function));

    FunctionCompiler::new(self.fnc.mc).compile(
      definition_pointer.clone(),
      Functionish::Fn(fn_.ident.clone(), fn_.function.clone()),
    );

    match capture_params.len() {
      0 => Value::Pointer(definition_pointer).to_ce(),
      _ => self.capturing_fn_ref(
        match fn_.ident {
          Some(ref ident) => ident.span,
          None => fn_.function.span,
        },
        fn_name,
        &Value::Pointer(definition_pointer),
        &capture_params,
        target_register,
      ),
    }
  }

  pub fn arrow_expression(
    &mut self,
    arrow_expr: &swc_ecma_ast::ArrowExpr,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let definition_pointer = self.fnc.allocate_defn_numbered("_anon");

    let capture_params = self
      .fnc
      .mc
      .scope_analysis
      .get_register_captures(&OwnerId::Span(arrow_expr.span));

    FunctionCompiler::new(self.fnc.mc).compile(
      definition_pointer.clone(),
      Functionish::Arrow(arrow_expr.clone()),
    );

    match capture_params.len() {
      0 => Value::Pointer(definition_pointer).to_ce(),
      _ => self.capturing_fn_ref(
        arrow_expr.span,
        None,
        &Value::Pointer(definition_pointer),
        &capture_params,
        target_register,
      ),
    }
  }

  pub fn capturing_fn_ref(
    &mut self,
    span: swc_common::Span,
    fn_name: Option<String>,
    fn_value: &Value,
    captures: &Vec<NameId>,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<Register>::new();

    let reg = match target_register {
      None => {
        let alloc_reg = match &fn_name {
          Some(name) => self.fnc.allocate_reg(name),
          None => self.fnc.allocate_numbered_reg("_anon"),
        };

        nested_registers.push(alloc_reg.clone());

        alloc_reg
      }
      Some(tr) => tr,
    };

    let mut bind_values = Array::default();

    for cap in captures {
      let cap_reg = match self.fnc.lookup_by_name_id(cap) {
        Some(v) => match v {
          Value::Register(r) => r,
          _ => continue,
        },
        None => {
          self.internal_error(
            cap.span(),
            &format!(
              "Failed to find capture {:?} for scope {:?}",
              cap, self.fnc.owner_id
            ),
          );

          continue;
        }
      };

      // If the capture is a parameter, it's excluded from TDZ checking. This is because TDZ applies
      // to let/const, and finding a parameter match means that the capture was already bound
      // elsewhere, so it's already been TDZ checked (checking here isn't just duplication, it's can
      // produce incorrect results, see captureShadowed.ts).
      let mut is_param = false;

      for p in &self.fnc.fn_.parameters {
        if p == &cap_reg {
          is_param = true;
        }
      }

      if !is_param {
        let cap_name = self
          .fnc
          .mc
          .scope_analysis
          .names
          .get(cap)
          .expect("Failed to find name");

        if let Some(tdz_end) = cap_name.tdz_end {
          if span.lo() <= tdz_end {
            self.error(
              span,
              &match &fn_name {
                Some(name) => format!(
                  "Referencing {} is invalid because it binds {} before its declaration (temporal \
                  dead zone)",
                  name, cap_name.sym,
                ),
                None => format!(
                  "Expression is invalid because capturing {} binds its value before its \
                  declaration (temporal dead zone)",
                  cap_name.sym,
                ),
              },
            );
          }
        }
      }

      bind_values
        .values
        .push(match self.fnc.lookup_by_name_id(cap) {
          Some(v) => match v {
            Value::Register(_) => v,
            _ => continue,
          },
          None => {
            self.internal_error(
              cap.span(),
              &format!(
                "Failed to find capture {:?} for scope {:?}",
                cap, self.fnc.owner_id
              ),
            );

            continue;
          }
        });
    }

    self.fnc.push(Instruction::Bind(
      fn_value.clone(),
      Value::Array(Box::new(bind_values)),
      reg.clone(),
    ));

    CompiledExpression::new(Value::Register(reg), nested_registers)
  }

  pub fn template_literal(
    &mut self,
    tpl: &swc_ecma_ast::Tpl,
    _target_register: Option<Register>,
  ) -> CompiledExpression {
    let len = tpl.exprs.len();

    assert_eq!(tpl.quasis.len(), len + 1);

    if len == 0 {
      return Value::String(tpl.quasis[0].raw.to_string()).to_ce();
    }

    let mut nested_registers = Vec::<Register>::new();

    let acc_reg = {
      let reg = self.fnc.allocate_tmp();
      nested_registers.push(reg.clone());

      reg
    };

    let first_expr = self.compile(&tpl.exprs[0], None);

    self.fnc.push(Instruction::OpPlus(
      Value::String(tpl.quasis[0].raw.to_string()),
      first_expr.value.clone(),
      acc_reg.clone(),
    ));

    self.fnc.release_ce(first_expr);

    for i in 1..len {
      self.fnc.push(Instruction::OpPlus(
        Value::Register(acc_reg.clone()),
        Value::String(tpl.quasis[i].raw.to_string()),
        acc_reg.clone(),
      ));

      let expr_i = self.compile(&tpl.exprs[i], None);

      self.fnc.push(Instruction::OpPlus(
        Value::Register(acc_reg.clone()),
        expr_i.value.clone(),
        acc_reg.clone(),
      ));

      self.fnc.release_ce(expr_i);
    }

    let last_str = tpl.quasis[len].raw.to_string();

    if !last_str.is_empty() {
      self.fnc.push(Instruction::OpPlus(
        Value::Register(acc_reg.clone()),
        Value::String(last_str),
        acc_reg.clone(),
      ));
    }

    CompiledExpression::new(Value::Register(acc_reg), nested_registers)
  }

  pub fn yield_expr(
    &mut self,
    yield_expr: &swc_ecma_ast::YieldExpr,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<Register>::new();

    let arg_compiled = match &yield_expr.arg {
      Some(arg) => self.compile(arg, None),
      None => CompiledExpression::empty(),
    };

    let dst = match target_register {
      Some(t) => t,
      None => {
        let tmp = self.fnc.allocate_tmp();
        nested_registers.push(tmp.clone());
        tmp
      }
    };

    self.fnc.push(match yield_expr.delegate {
      false => Instruction::Yield(arg_compiled.value.clone(), dst.clone()),
      true => Instruction::YieldStar(arg_compiled.value.clone(), dst.clone()),
    });

    self.fnc.release_ce(arg_compiled);

    CompiledExpression::new(Value::Register(dst), nested_registers)
  }

  pub fn jsx_element(
    &mut self,
    jsx_element: &swc_ecma_ast::JSXElement,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let tag = self.get_tag(&jsx_element.opening.name);

    if let Some(closing) = &jsx_element.closing {
      if self.get_tag(&closing.name) != tag {
        self.error(
          closing.name.span(),
          "JSX closing tag does not match opening tag",
        );
      }
    }

    self.jsx(
      Some(tag),
      &jsx_element.opening.attrs,
      &jsx_element.children,
      target_register,
    )
  }

  pub fn jsx_fragment(
    &mut self,
    jsx_fragment: &swc_ecma_ast::JSXFragment,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    self.jsx(None, &vec![], &jsx_fragment.children, target_register)
  }

  fn jsx(
    &mut self,
    tag: Option<String>,
    jsx_attrs: &Vec<swc_ecma_ast::JSXAttrOrSpread>,
    jsx_children: &Vec<swc_ecma_ast::JSXElementChild>,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let mut sub_nested_registers = Vec::<Register>::new();

    let mut attrs = Vec::<Value>::new();

    for attr in jsx_attrs {
      match attr {
        swc_ecma_ast::JSXAttrOrSpread::JSXAttr(attr) => {
          let key = match &attr.name {
            swc_ecma_ast::JSXAttrName::Ident(ident) => Value::String(ident.sym.to_string()),
            swc_ecma_ast::JSXAttrName::JSXNamespacedName(_) => {
              self.todo(attr.name.span(), "JSXNamespacedName attribute name");
              Value::String("(error)".to_owned())
            }
          };

          let mut compiled_value = match &attr.value {
            Some(value) => match value {
              swc_ecma_ast::JSXAttrValue::Lit(lit) => self.compile_literal(lit).to_ce(),
              swc_ecma_ast::JSXAttrValue::JSXExprContainer(jsx_expr_container) => {
                match &jsx_expr_container.expr {
                  swc_ecma_ast::JSXExpr::JSXEmptyExpr(empty) => {
                    self.error(empty.span, "Empty jsx expression");
                    CompiledExpression::empty()
                  }
                  swc_ecma_ast::JSXExpr::Expr(expr) => self.compile(expr, None),
                }
              }
              swc_ecma_ast::JSXAttrValue::JSXElement(el) => self.jsx_element(el, None),
              swc_ecma_ast::JSXAttrValue::JSXFragment(fragment) => {
                self.jsx_fragment(fragment, None)
              }
            },
            None => CompiledExpression::empty(),
          };

          attrs.push(Value::Array(Box::new(Array {
            values: vec![key, compiled_value.value],
          })));

          sub_nested_registers.append(&mut compiled_value.nested_registers);
          compiled_value.release_checker.has_unreleased_registers = false;
        }
        swc_ecma_ast::JSXAttrOrSpread::SpreadElement(_) => {
          self.todo(attr.span(), "JSXSpreadElement");
        }
      }
    }

    let mut children = Vec::<Value>::new();

    for child in jsx_children {
      let mut compiled_child = match child {
        swc_ecma_ast::JSXElementChild::JSXText(text) => {
          Value::String(text.value.to_string()).to_ce()
        }
        swc_ecma_ast::JSXElementChild::JSXExprContainer(jsx_expr_container) => {
          match &jsx_expr_container.expr {
            swc_ecma_ast::JSXExpr::JSXEmptyExpr(empty) => {
              self.error(empty.span, "Empty jsx expression");
              CompiledExpression::empty()
            }
            swc_ecma_ast::JSXExpr::Expr(expr) => self.compile(expr, None),
          }
        }
        swc_ecma_ast::JSXElementChild::JSXSpreadChild(_) => {
          self.todo(child.span(), "JSXSpreadChild");
          CompiledExpression::empty()
        }
        swc_ecma_ast::JSXElementChild::JSXElement(el) => self.jsx_element(el, None),
        swc_ecma_ast::JSXElementChild::JSXFragment(fragment) => self.jsx_fragment(fragment, None),
      };

      children.push(compiled_child.value);
      sub_nested_registers.append(&mut compiled_child.nested_registers);
      compiled_child.release_checker.has_unreleased_registers = false;
    }

    let mut nested_registers = Vec::<Register>::new();

    let dest = match target_register {
      Some(tr) => tr,
      None => {
        let tmp = self.fnc.allocate_tmp();
        nested_registers.push(tmp.clone());
        tmp
      }
    };

    self.fnc.push(Instruction::Jsx(
      match tag {
        Some(tag_str) => Value::String(tag_str),
        None => Value::Void,
      },
      Value::Array(Box::new(Array { values: attrs })),
      Value::Array(Box::new(Array { values: children })),
      dest.clone(),
    ));

    for reg in sub_nested_registers {
      self.fnc.release_reg(&reg);
    }

    CompiledExpression::new(Value::Register(dest), nested_registers)
  }

  pub fn ident(
    &mut self,
    ident: &CrateIdent,
    target_register: Option<Register>,
  ) -> CompiledExpression {
    let fn_as_owner_id = match self.fnc.mc.scope_analysis.lookup(ident) {
      Some(name) => match name.type_ == NameType::Function {
        true => match name.id {
          // TODO: This is a bit of a hack, it might break...
          // functions have an owner id, and the name id should
          // have the same span... at least it does now
          NameId::Span(span) => Some(OwnerId::Span(span)),
          _ => None, // Internal error?
        },
        false => None,
      },
      _ => {
        self.internal_error(
          ident.span,
          &format!("Failed to lookup identifier `{}`", ident.sym),
        );

        None
      }
    };

    let value = self.fnc.lookup_value(ident).unwrap_or_default();

    let name = match self.fnc.lookup(ident) {
      Some(v) => v,
      None => {
        return Value::Undefined.to_ce();
      }
    };

    match fn_as_owner_id {
      Some(owner_id) => {
        let capture_params = self.fnc.mc.scope_analysis.get_register_captures(&owner_id);

        match capture_params.len() {
          0 => value.to_ce(),
          _ => self.capturing_fn_ref(
            ident.span,
            Some(ident.sym.to_string()),
            &value,
            &capture_params,
            target_register,
          ),
        }
      }
      None => match value {
        Value::Register(reg) => {
          if name.mutations.is_empty() {
            // Just use the register for the variable if it's not mutated
            return Value::Register(reg).to_ce();
          }

          // Otherwise, we need to capture the current value for the result of the expression
          let new_reg = self.fnc.allocate_tmp();

          self
            .fnc
            .push(Instruction::Mov(Value::Register(reg), new_reg.clone()));

          CompiledExpression::new(Value::Register(new_reg.clone()), vec![new_reg])
        }
        _ => value.to_ce(),
      },
    }
  }

  pub fn compile_literal(&mut self, lit: &swc_ecma_ast::Lit) -> Value {
    match value_from_literal(lit) {
      Ok(value) => value,
      Err(err) => {
        self.todo(lit.span(), err);
        Value::Register(self.fnc.allocate_numbered_reg("_todo_unsupported_literal"))
      }
    }
  }

  pub fn pat(&mut self, pat: &swc_ecma_ast::Pat, register: &Register, skip_release: bool) {
    use swc_ecma_ast::Pat;

    match pat {
      Pat::Ident(ident) => {
        let ident_reg = self.fnc.get_pattern_register(pat);

        if register != &ident_reg {
          self.internal_error(
            pat.span(),
            &format!(
              "Register mismatch for parameter {} (expected {}, got {})",
              ident.id.sym,
              Structured(&ident_reg),
              Structured(register),
            ),
          );

          // Note: We still have this sensible interpretation, so emitting it
          // may help troubleshooting the error above. Hopefully it never
          // occurs.
          self.fnc.push(Instruction::Mov(
            Value::Register(register.clone()),
            ident_reg,
          ));
        }
      }
      Pat::Assign(assign) => self.assign_pat(assign, register),
      Pat::Array(array) => {
        for (i, elem_opt) in array.elems.iter().enumerate() {
          let elem = match elem_opt {
            Some(elem) => elem,
            None => continue,
          };

          let elem_reg = self.fnc.get_pattern_register(elem);

          self.fnc.push(Instruction::Sub(
            Value::Register(register.clone()),
            Value::Number(Number(i as f64)),
            elem_reg.clone(),
          ));

          self.pat(elem, &elem_reg, false);
        }

        if !skip_release {
          self.fnc.release_reg(register);
        }
      }
      Pat::Object(object) => {
        for prop in &object.props {
          use swc_ecma_ast::ObjectPatProp;

          match prop {
            ObjectPatProp::KeyValue(kv) => {
              let param_reg = self.fnc.get_pattern_register(&kv.value);
              let compiled_key = self.prop_name(&kv.key);

              self.fnc.push(Instruction::Sub(
                Value::Register(register.clone()),
                compiled_key.value.clone(),
                param_reg.clone(),
              ));

              self.fnc.release_ce(compiled_key);

              self.pat(&kv.value, &param_reg, false);
            }
            ObjectPatProp::Assign(assign) => {
              let key = assign.key.sym.to_string();
              let reg = self
                .fnc
                .get_variable_register(&CrateIdent::from_swc_ident(&assign.key));

              self.fnc.push(Instruction::Sub(
                Value::Register(register.clone()),
                Value::String(key),
                reg.clone(),
              ));

              if let Some(value) = &assign.value {
                self.default_expr(value, &reg);
              }
            }
            ObjectPatProp::Rest(rest) => {
              self.todo(rest.span, "Rest pattern in object destructuring");
            }
          }
        }

        if !skip_release {
          self.fnc.release_reg(register);
        }
      }
      Pat::Invalid(_) => {
        // Diagnostic emitted elsewhere
      }
      Pat::Rest(_) => {
        // TODO (Diagnostic emitted elsewhere)
      }
      Pat::Expr(expr) => {
        let mut at = TargetAccessor::compile(self, expr, true);
        at.assign_and_packup(self, &Value::Register(register.clone()), false);
      }
    }
  }

  pub fn assign_pat(&mut self, assign: &swc_ecma_ast::AssignPat, register: &Register) {
    use swc_ecma_ast::Pat;

    if let Pat::Expr(expr) = &*assign.left {
      let mut at = TargetAccessor::compile(self, expr, true);
      self.default_expr(&assign.right, register);
      at.assign_and_packup(self, &Value::Register(register.clone()), false);
    } else {
      self.default_expr(&assign.right, register);
      self.pat(&assign.left, register, false);
    }
  }

  fn default_expr(&mut self, expr: &swc_ecma_ast::Expr, register: &Register) {
    let provided_reg = self.fnc.allocate_tmp();

    let initialized_label = Label {
      name: self
        .fnc
        .label_allocator
        .allocate(&format!("{}_initialized", register.name)),
    };

    self.fnc.push(Instruction::OpTripleNe(
      Value::Register(register.clone()),
      Value::Undefined,
      provided_reg.clone(),
    ));

    self.fnc.push(Instruction::JmpIf(
      Value::Register(provided_reg.clone()),
      initialized_label.ref_(),
    ));

    self.fnc.release_reg(&provided_reg);

    self.compile_into(expr, register.clone());

    self.fnc.label(initialized_label);
  }

  fn args<'b, IterT>(
    &mut self,
    arg_list: IterT,
    target_register: Option<Register>,
  ) -> CompiledExpression
  where
    IterT: Iterator<Item = Option<&'b swc_ecma_ast::ExprOrSpread>>,
  {
    let mut segments = Vec::<Value>::new();
    let mut current = Vec::<Value>::new();
    let mut sub_nested_registers = Vec::<Register>::new();

    for arg in arg_list {
      let arg = match arg {
        None => {
          current.push(Value::Void);
          continue;
        }
        Some(arg) => arg,
      };

      if arg.spread.is_some() && !current.is_empty() {
        segments.push(Value::Array(Box::new(Array {
          values: take(&mut current),
        })));
      }

      let mut compiled_elem = self.compile(&arg.expr, None);

      if arg.spread.is_some() {
        segments.push(compiled_elem.value);
      } else {
        current.push(compiled_elem.value);
      }

      sub_nested_registers.append(&mut compiled_elem.nested_registers);
      compiled_elem.release_checker.has_unreleased_registers = false;
    }

    if segments.is_empty() {
      return match target_register {
        None => CompiledExpression::new(
          Value::Array(Box::new(Array { values: current })),
          sub_nested_registers,
        ),
        Some(tr) => {
          self.fnc.push(Instruction::Mov(
            Value::Array(Box::new(Array { values: current })),
            tr.clone(),
          ));

          for reg in sub_nested_registers {
            self.fnc.release_reg(&reg);
          }

          CompiledExpression::new(Value::Register(tr), vec![])
        }
      };
    }

    if !current.is_empty() {
      segments.push(Value::Array(Box::new(Array { values: current })));
    }

    let mut nested_registers = Vec::<Register>::new();

    let res_reg = match target_register {
      Some(target_register) => target_register,
      None => {
        let tmp = self.fnc.allocate_tmp();
        nested_registers.push(tmp.clone());
        tmp
      }
    };

    self.fnc.push(Instruction::Cat(
      Value::Array(Box::new(Array { values: segments })),
      res_reg.clone(),
    ));

    for reg in sub_nested_registers {
      self.fnc.release_reg(&reg);
    }

    CompiledExpression::new(Value::Register(res_reg), nested_registers)
  }

  pub fn get_tag(&mut self, jsx_element_name: &swc_ecma_ast::JSXElementName) -> String {
    match jsx_element_name {
      swc_ecma_ast::JSXElementName::Ident(ident) => ident.sym.to_string(),
      swc_ecma_ast::JSXElementName::JSXMemberExpr(member_expr) => {
        self.todo(member_expr.span(), "JSXMemberExpr");
        "(error)".to_string()
      }
      swc_ecma_ast::JSXElementName::JSXNamespacedName(namespaced_name) => {
        self.todo(namespaced_name.span(), "JSXNamespacedName");
        "(error)".to_string()
      }
    }
  }
}

pub enum UnaryOpType {
  Ordinary(OrdinaryUnaryOp),
  Void,
  Delete,
}

impl UnaryOpType {
  fn from_unary_op(op: swc_ecma_ast::UnaryOp) -> Self {
    match op {
      swc_ecma_ast::UnaryOp::Minus => UnaryOpType::Ordinary(OrdinaryUnaryOp::Minus),
      swc_ecma_ast::UnaryOp::Plus => UnaryOpType::Ordinary(OrdinaryUnaryOp::Plus),
      swc_ecma_ast::UnaryOp::Bang => UnaryOpType::Ordinary(OrdinaryUnaryOp::Bang),
      swc_ecma_ast::UnaryOp::Tilde => UnaryOpType::Ordinary(OrdinaryUnaryOp::Tilde),
      swc_ecma_ast::UnaryOp::TypeOf => UnaryOpType::Ordinary(OrdinaryUnaryOp::TypeOf),
      swc_ecma_ast::UnaryOp::Void => UnaryOpType::Void,
      swc_ecma_ast::UnaryOp::Delete => UnaryOpType::Delete,
    }
  }
}

pub enum OrdinaryUnaryOp {
  Minus,
  Plus,
  Bang,
  Tilde,
  TypeOf,
}

impl OrdinaryUnaryOp {
  fn make_instruction(self, arg: Value, dst: Register) -> Instruction {
    match self {
      OrdinaryUnaryOp::Minus => Instruction::UnaryMinus(arg, dst),
      OrdinaryUnaryOp::Plus => Instruction::UnaryPlus(arg, dst),
      OrdinaryUnaryOp::Bang => Instruction::OpNot(arg, dst),
      OrdinaryUnaryOp::Tilde => Instruction::OpBitNot(arg, dst),
      OrdinaryUnaryOp::TypeOf => Instruction::TypeOf(arg, dst),
    }
  }
}

pub fn make_binary_op(
  op: swc_ecma_ast::BinaryOp,
  arg1: Value,
  arg2: Value,
  dst: Register,
) -> Instruction {
  use swc_ecma_ast::BinaryOp::*;

  match op {
    EqEq => Instruction::OpEq(arg1, arg2, dst),
    NotEq => Instruction::OpNe(arg1, arg2, dst),
    EqEqEq => Instruction::OpTripleEq(arg1, arg2, dst),
    NotEqEq => Instruction::OpTripleNe(arg1, arg2, dst),
    Lt => Instruction::OpLess(arg1, arg2, dst),
    LtEq => Instruction::OpLessEq(arg1, arg2, dst),
    Gt => Instruction::OpGreater(arg1, arg2, dst),
    GtEq => Instruction::OpGreaterEq(arg1, arg2, dst),
    LShift => Instruction::OpLeftShift(arg1, arg2, dst),
    RShift => Instruction::OpRightShift(arg1, arg2, dst),
    ZeroFillRShift => Instruction::OpRightShiftUnsigned(arg1, arg2, dst),
    Add => Instruction::OpPlus(arg1, arg2, dst),
    Sub => Instruction::OpMinus(arg1, arg2, dst),
    Mul => Instruction::OpMul(arg1, arg2, dst),
    Div => Instruction::OpDiv(arg1, arg2, dst),
    Mod => Instruction::OpMod(arg1, arg2, dst),
    BitOr => Instruction::OpBitOr(arg1, arg2, dst),
    BitXor => Instruction::OpBitXor(arg1, arg2, dst),
    BitAnd => Instruction::OpBitAnd(arg1, arg2, dst),
    LogicalOr => Instruction::OpOr(arg1, arg2, dst),
    LogicalAnd => Instruction::OpAnd(arg1, arg2, dst),
    In => Instruction::In(arg1, arg2, dst),
    InstanceOf => Instruction::InstanceOf(arg1, arg2, dst),
    Exp => Instruction::OpExp(arg1, arg2, dst),
    NullishCoalescing => Instruction::OpNullishCoalesce(arg1, arg2, dst),
  }
}

pub fn get_binary_op_for_assign_op(
  assign_op: swc_ecma_ast::AssignOp,
) -> Option<swc_ecma_ast::BinaryOp> {
  use swc_ecma_ast::AssignOp;
  use swc_ecma_ast::BinaryOp;

  match assign_op {
    AssignOp::Assign => None,
    AssignOp::AddAssign => Some(BinaryOp::Add),
    AssignOp::SubAssign => Some(BinaryOp::Sub),
    AssignOp::MulAssign => Some(BinaryOp::Mul),
    AssignOp::DivAssign => Some(BinaryOp::Div),
    AssignOp::ModAssign => Some(BinaryOp::Mod),
    AssignOp::LShiftAssign => Some(BinaryOp::LShift),
    AssignOp::RShiftAssign => Some(BinaryOp::RShift),
    AssignOp::ZeroFillRShiftAssign => Some(BinaryOp::ZeroFillRShift),
    AssignOp::BitOrAssign => Some(BinaryOp::BitOr),
    AssignOp::BitXorAssign => Some(BinaryOp::BitXor),
    AssignOp::BitAndAssign => Some(BinaryOp::BitAnd),
    AssignOp::ExpAssign => Some(BinaryOp::Exp),
    AssignOp::AndAssign => Some(BinaryOp::LogicalAnd),
    AssignOp::OrAssign => Some(BinaryOp::LogicalOr),
    AssignOp::NullishAssign => Some(BinaryOp::NullishCoalescing),
  }
}

pub fn make_update_op(op: swc_ecma_ast::UpdateOp, register: Register) -> Instruction {
  use swc_ecma_ast::UpdateOp::*;

  match op {
    PlusPlus => Instruction::OpInc(register),
    MinusMinus => Instruction::OpDec(register),
  }
}

pub fn value_from_literal(lit: &swc_ecma_ast::Lit) -> Result<Value, &'static str> {
  use swc_ecma_ast::Lit::*;

  Ok(match lit {
    Str(str_) => Value::String(str_.value.to_string()),
    Bool(bool_) => Value::Bool(bool_.value),
    Null(_) => Value::Null,
    Num(num) => Value::Number(Number(num.value)),
    BigInt(bigint) => Value::BigInt(bigint.value.clone()),
    Regex(_) => return Err("Regex literals"),
    JSXText(_) => return Err("JSXText literals"),
  })
}
