use super::scope::{Scope, ScopeTrait, MappedName};
use super::function_compiler::FunctionCompiler;

pub struct CompiledExpression {
  pub value_assembly: String,
  pub nested_registers: Vec<String>,
}

pub struct ExpressionCompiler<'a> {
  pub scope: &'a Scope,
  pub fnc: &'a mut FunctionCompiler,
}

impl<'a> ExpressionCompiler<'a> {
  pub fn compile(
    &mut self,
    expr: &swc_ecma_ast::Expr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    use swc_ecma_ast::Expr::*;

    match expr {
      This(_) => {
        return self.inline("%this".to_string(), target_register);
      },
      Array(array_exp) => {
        return self.array_expression(array_exp, target_register);
      },
      Object(object_exp) => {
        return self.object_expression(object_exp, target_register);
      },
      Fn(_) => std::panic!("Not implemented: Fn expression"),
      Unary(un_exp) => {
        return self.unary_expression(un_exp, target_register);
      },
      Update(_) => std::panic!("Not implemented: Update expression"),
      Bin(bin_exp) => {
        return self.binary_expression(bin_exp, target_register);
      },
      Assign(assign_exp) => {
        return self.assign_expression(assign_exp, target_register);
      },
      Member(member_exp) => {
        return self.member_expression(member_exp, target_register);
      },
      SuperProp(_) => std::panic!("Not implemented: SuperProp expression"),
      Cond(_) => std::panic!("Not implemented: Cond expression"),
      Call(call_exp) => {
        return self.call_expression(call_exp, target_register);
      },
      New(_) => std::panic!("Not implemented: New expression"),
      Seq(_) => std::panic!("Not implemented: Seq expression"),
      Ident(ident) => {
        return self.identifier(ident, target_register);
      },
      Lit(lit) => {
        return self.literal(lit, target_register);
      },
      Tpl(_) => std::panic!("Not implemented: Tpl expression"),
      TaggedTpl(_) => std::panic!("Not implemented: TaggedTpl expression"),
      Arrow(_) => std::panic!("Not implemented: Arrow expression"),
      Class(_) => std::panic!("Not implemented: Class expression"),
      Yield(_) => std::panic!("Not implemented: Yield expression"),
      MetaProp(_) => std::panic!("Not implemented: MetaProp expression"),
      Await(_) => std::panic!("Not implemented: Await expression"),
      Paren(p) => {
        return self.compile(&*p.expr, target_register);
      },
      JSXMember(_) => std::panic!("Not implemented: JSXMember expression"),
      JSXNamespacedName(_) => std::panic!("Not implemented: JSXNamespacedName expression"),
      JSXEmpty(_) => std::panic!("Not implemented: JSXEmpty expression"),
      JSXElement(_) => std::panic!("Not implemented: JSXElement expression"),
      JSXFragment(_) => std::panic!("Not implemented: JSXFragment expression"),
      TsTypeAssertion(_) => std::panic!("Not implemented: TsTypeAssertion expression"),
      TsConstAssertion(_) => std::panic!("Not implemented: TsConstAssertion expression"),
      TsNonNull(_) => std::panic!("Not implemented: TsNonNull expression"),
      TsAs(_) => std::panic!("Not implemented: TsAs expression"),
      TsInstantiation(_) => std::panic!("Not implemented: TsInstantiation expression"),
      PrivateName(_) => std::panic!("Not implemented: PrivateName expression"),
      OptChain(_) => std::panic!("Not implemented: OptChain expression"),
      Invalid(_) => std::panic!("Not implemented: Invalid expression"),
    };
  }

  pub fn unary_expression(
    &mut self,
    un_exp: &swc_ecma_ast::UnaryExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<String>::new();

    let arg = self.compile(
      &un_exp.arg,
      None,
    );

    let mut instr = "  ".to_string();
    instr += get_unary_op_str(un_exp.op);
    instr += " ";
    instr += &arg.value_assembly;

    for used_reg in arg.nested_registers {
      self.fnc.reg_allocator.release(&used_reg);
    }

    let target: String = match &target_register {
      None => {
        let res = self.fnc.reg_allocator.allocate_numbered(&"_tmp".to_string());
        nested_registers.push(res.clone());
        res
      },
      Some(t) => t.clone(),
    };

    instr += " %";
    instr += &target;

    self.fnc.definition.push(instr);

    return CompiledExpression {
      value_assembly: std::format!("%{}", target),
      nested_registers: nested_registers,
    };
  }

  pub fn binary_expression(
    &mut self,
    bin: &swc_ecma_ast::BinExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<String>::new();

    let left = self.compile(
      &bin.left,
      None
    );

    let right = self.compile(
      &bin.right,
      None,
    );

    let mut instr = "  ".to_string();

    // FIXME: && and || need to avoid executing the right side where applicable
    // (mandatory if they mutate)
    instr += get_binary_op_str(bin.op);

    instr += " ";
    instr += &left.value_assembly;
    instr += " ";
    instr += &right.value_assembly;

    for used_reg in left.nested_registers {
      self.fnc.reg_allocator.release(&used_reg);
    }

    for used_reg in right.nested_registers {
      self.fnc.reg_allocator.release(&used_reg);
    }

    let target: String = match &target_register {
      None => {
        let res = self.fnc.reg_allocator.allocate_numbered(&"_tmp".to_string());
        nested_registers.push(res.clone());
        res
      },
      Some(t) => t.clone(),
    };

    instr += " %";
    instr += &target;

    self.fnc.definition.push(instr);

    return CompiledExpression {
      value_assembly: std::format!("%{}", target),
      nested_registers: nested_registers,
    };
  }

  pub fn assign_expression(
    &mut self,
    assign_exp: &swc_ecma_ast::AssignExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    if assign_exp.op != swc_ecma_ast::AssignOp::Assign {
      std::panic!("Not implemented: compound assignment");
    }

    let assign_name = match &assign_exp.left {
      swc_ecma_ast::PatOrExpr::Expr(_) => std::panic!("Not implemented: assign to expr"),
      swc_ecma_ast::PatOrExpr::Pat(pat) => match &**pat {
        swc_ecma_ast::Pat::Ident(ident) => ident.id.sym.to_string(),
        _ => std::panic!("Not implemented: destructuring"),
      },
    };

    let assign_register = match self.scope.get(&assign_name) {
      None => std::panic!("Unresolved reference"),
      Some(mapping) => match mapping {
        MappedName::Definition(_) => std::panic!("Invalid: assignment to definition"),
        MappedName::Register(reg_name) => reg_name,
      }
    };

    let rhs = self.compile(
      &*assign_exp.right,
      Some(assign_register.clone()),
    );

    // TODO: Consider making two variations of compile, one that takes a target
    // register and one that doesn't. This may simplify things eg by not
    // returning any nested registers when there's a target.
    assert_eq!(rhs.nested_registers.len(), 0);

    if target_register.is_some() {
      let tr = target_register.unwrap();

      let mut instr = "  mov %".to_string();
      instr += &assign_register;
      instr += " %";
      instr += &tr;
      self.fnc.definition.push(instr);
    }

    return CompiledExpression {
      value_assembly: "%".to_string() + &assign_register,
      nested_registers: Vec::new(),
    };
  }

  pub fn array_expression(
    &mut self,
    array_exp: &swc_ecma_ast::ArrayLit,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut value_assembly = "[".to_string();
    let mut sub_nested_registers = Vec::<String>::new();

    for i in 0..array_exp.elems.len() {
      match &array_exp.elems[i] {
        None => {
          value_assembly += "void";
        },
        Some(elem) => {
          if elem.spread.is_some() {
            std::panic!("Not implemented: spread expression");
          }

          let mut compiled_elem = self.compile(&*elem.expr, None);
          value_assembly += &compiled_elem.value_assembly;
          sub_nested_registers.append(&mut compiled_elem.nested_registers);
        },
      }

      if i != array_exp.elems.len() - 1 {
        value_assembly += ", ";
      }
    }

    value_assembly += "]";

    return match target_register {
      None => CompiledExpression {
        value_assembly: value_assembly,
        nested_registers: sub_nested_registers,
      },
      Some(tr) => {
        self.fnc.definition.push(
          std::format!("  mov {} %{}", value_assembly, tr)
        );

        for reg in sub_nested_registers {
          self.fnc.reg_allocator.release(&reg);
        }
        
        CompiledExpression {
          value_assembly: std::format!("%{}", tr),
          nested_registers: Vec::new(),
        }
      },
    };
  }

  pub fn object_expression(
    &mut self,
    object_exp: &swc_ecma_ast::ObjectLit,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut value_assembly = "{".to_string();
    let mut sub_nested_registers = Vec::<String>::new();

    for i in 0..object_exp.props.len() {
      match &object_exp.props[i] {
        swc_ecma_ast::PropOrSpread::Spread(_) => {
          std::panic!("Not implemented: spread expression");
        },
        swc_ecma_ast::PropOrSpread::Prop(prop) => match &**prop {
          swc_ecma_ast::Prop::Shorthand(_) => std::panic!("Not implemented: Shorthand prop"),
          swc_ecma_ast::Prop::KeyValue(kv) => {
            let key_assembly = match &kv.key {
              swc_ecma_ast::PropName::Ident(ident) =>
                std::format!("\"{}\"", ident.sym.to_string())
              ,
              swc_ecma_ast::PropName::Str(str_) =>
                // TODO: Escaping
                std::format!("\"{}\"", str_.value.to_string())
              ,
              swc_ecma_ast::PropName::Num(num) =>
                // TODO: JS number stringification (different from rust)
                std::format!("\"{}\"", num.value.to_string())
              ,
              swc_ecma_ast::PropName::Computed(comp) => {
                // TODO: Always using a register is maybe not ideal
                // At the least, the assembly supports definitions and should
                // maybe support any value here
                let reg = self.fnc.reg_allocator.allocate_numbered(&"computed_key".to_string());
                let compiled = self.compile(&comp.expr, Some(reg.clone()));
                assert_eq!(compiled.nested_registers.len(), 0);
                sub_nested_registers.push(reg.clone());

                std::format!("%{}", reg)
              },
              swc_ecma_ast::PropName::BigInt(bigint) =>
                std::format!("\"{}\"", bigint.value.to_string())
              ,
            };

            value_assembly += &key_assembly;
            value_assembly += ": ";

            let mut compiled_value = self.compile(&kv.value, None);
            sub_nested_registers.append(&mut compiled_value.nested_registers);
            value_assembly += &compiled_value.value_assembly;
          },
          swc_ecma_ast::Prop::Assign(_) => std::panic!("Not implemented: Assign prop"),
          swc_ecma_ast::Prop::Getter(_) => std::panic!("Not implemented: Getter prop"),
          swc_ecma_ast::Prop::Setter(_) => std::panic!("Not implemented: Setter prop"),
          swc_ecma_ast::Prop::Method(_) => std::panic!("Not implemented: Method prop"),
        },
      }

      if i != object_exp.props.len() - 1 {
        value_assembly += ", ";
      }
    }

    value_assembly += "}";

    return match target_register {
      None => CompiledExpression {
        value_assembly: value_assembly,
        nested_registers: sub_nested_registers,
      },
      Some(tr) => {
        self.fnc.definition.push(
          std::format!("  mov {} %{}", value_assembly, tr)
        );

        for reg in sub_nested_registers {
          self.fnc.reg_allocator.release(&reg);
        }
        
        CompiledExpression {
          value_assembly: std::format!("%{}", tr),
          nested_registers: Vec::new(),
        }
      },
    };
  }

  pub fn member_expression(
    &mut self,
    member_exp: &swc_ecma_ast::MemberExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let compiled_obj = self.compile(&member_exp.obj, None);
    
    let mut sub_instr = "  sub ".to_string();
    sub_instr += &compiled_obj.value_assembly;

    let compiled_prop = match &member_exp.prop {
      swc_ecma_ast::MemberProp::Ident(ident) => CompiledExpression {
        value_assembly: format!("\"{}\"", ident.sym.to_string()),
        nested_registers: Vec::new(),
      },
      swc_ecma_ast::MemberProp::Computed(computed) => {
        self.compile(&computed.expr, None)
      },
      swc_ecma_ast::MemberProp::PrivateName(_) => {
        std::panic!("Not implemented: private name");
      },
    };

    sub_instr += " ";
    sub_instr += &compiled_prop.value_assembly;

    for reg in compiled_obj.nested_registers {
      self.fnc.reg_allocator.release(&reg);
    }

    for reg in compiled_prop.nested_registers {
      self.fnc.reg_allocator.release(&reg);
    }

    let mut nested_registers = Vec::<String>::new();

    let dest = match &target_register {
      Some(tr) => ("%".to_string() + &tr),
      None => {
        let reg = self.fnc.reg_allocator.allocate_numbered(&"_tmp".to_string());
        nested_registers.push(reg.clone());

        "%".to_string() + &reg
      },
    };

    sub_instr += " ";
    sub_instr += &dest;

    self.fnc.definition.push(sub_instr);

    return CompiledExpression {
      value_assembly: dest,
      nested_registers: nested_registers,
    };
  }

  pub fn call_expression(
    &mut self,
    call_exp: &swc_ecma_ast::CallExpr,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let mut nested_registers = Vec::<String>::new();
    let mut sub_nested_registers = Vec::<String>::new();

    let mut callee = match &call_exp.callee {
      swc_ecma_ast::Callee::Expr(expr) => self.compile(&*expr, None),
      _ => std::panic!("Not implemented: non-expression callee"),
    };

    sub_nested_registers.append(&mut callee.nested_registers);

    let mut instr = "  call ".to_string();
    instr += &callee.value_assembly;
    instr += " [";

    for i in 0..call_exp.args.len() {
      let arg = &call_exp.args[i];

      if arg.spread.is_some() {
        std::panic!("Not implemented: argument spreading");
      }

      let mut compiled_arg = self.compile(&*arg.expr, None);
      sub_nested_registers.append(&mut compiled_arg.nested_registers);

      instr += &compiled_arg.value_assembly;

      if i != call_exp.args.len() - 1 {
        instr += ", ";
      }
    }

    instr += "] ";

    let dest = match &target_register {
      Some(tr) => ("%".to_string() + &tr),
      None => {
        let reg = self.fnc.reg_allocator.allocate_numbered(&"_tmp".to_string());
        nested_registers.push(reg.clone());

        "%".to_string() + &reg
      },
    };

    instr += &dest;

    self.fnc.definition.push(instr);

    for reg in sub_nested_registers {
      self.fnc.reg_allocator.release(&reg);
    }

    return CompiledExpression {
      value_assembly: dest,
      nested_registers: nested_registers,
    };
  }

  pub fn literal(
    &mut self,
    lit: &swc_ecma_ast::Lit,
    target_register: Option<String>,
  ) -> CompiledExpression {
    return self.inline(compile_literal(lit), target_register);
  }

  pub fn inline(
    &mut self,
    value_assembly: String,
    target_register: Option<String>,
  ) -> CompiledExpression {
    return match target_register {
      None => CompiledExpression {
        value_assembly: value_assembly,
        nested_registers: Vec::new(),
      },
      Some(t) => {
        let mut instr = "  mov ".to_string();
        instr += &value_assembly;
        instr += " %";
        instr += &t;
        self.fnc.definition.push(instr);

        CompiledExpression {
          value_assembly: std::format!("%{}", t),
          nested_registers: Vec::new(),
        }
      },
    };
  }

  pub fn identifier(
    &mut self,
    ident: &swc_ecma_ast::Ident,
    target_register: Option<String>,
  ) -> CompiledExpression {
    let ident_string = ident.sym.to_string();

    let mapped = self.scope.get(&ident_string).expect("Identifier not found in scope");

    let value_assembly = match mapped {
      MappedName::Register(reg) => "%".to_string() + &reg,
      MappedName::Definition(def) => "@".to_string() + &def,
    };

    return self.inline(value_assembly, target_register);
  }
}

pub fn compile_literal(lit: &swc_ecma_ast::Lit) -> String {
  use swc_ecma_ast::Lit::*;

  return match lit {
    Str(str_) => std::format!("\"{}\"", str_.value), // TODO: Escaping
    Bool(bool_) => bool_.value.to_string(),
    Null(_) => "null".to_string(),
    Num(num) => num.value.to_string(),
    BigInt(_) => std::panic!("Not implemented: BigInt expression"),
    Regex(_) => std::panic!("Not implemented: Regex expression"),
    JSXText(_) => std::panic!("Not implemented: JSXText expression"),
  };
}

pub fn get_binary_op_str(op: swc_ecma_ast::BinaryOp) -> &'static str {
  use swc_ecma_ast::BinaryOp::*;

  return match op {
    EqEq => "op==",
    NotEq => "op!=",
    EqEqEq => "op===",
    NotEqEq => "op!==",
    Lt => "op<",
    LtEq => "op<=",
    Gt => "op>",
    GtEq => "op>=",
    LShift => "op<<",
    RShift => "op>>",
    ZeroFillRShift => "op>>>",
    Add => "op+",
    Sub => "op-",
    Mul => "op*",
    Div => "op/",
    Mod => "op%",
    BitOr => "op|",
    BitXor => "op^",
    BitAnd => "op&",
    LogicalOr => "op||",
    LogicalAnd => "op&&",
    In => "in",
    InstanceOf => "instanceof",
    Exp => "op**",
    NullishCoalescing => "op??",
  };
}

pub fn get_unary_op_str(op: swc_ecma_ast::UnaryOp) -> &'static str {
  use swc_ecma_ast::UnaryOp::*;

  return match op {
    Minus => "unary-",
    Plus => "unary+",
    Bang => "op!",
    Tilde => "op~",
    TypeOf => "typeof",
    Void => std::panic!("No matching instruction"),
    Delete => std::panic!("No matching instruction"),
  };
}