use valuescript_vm::operations::to_i32;

use crate::{
  asm::{Array, Builtin, Number, Object, Value},
  expression_compiler::value_from_literal,
  ident::Ident,
  module_compiler::ModuleCompiler,
};

pub fn static_eval_expr(mc: &ModuleCompiler, expr: &swc_ecma_ast::Expr) -> Option<Value> {
  let symbol_iterator_opt = as_symbol_iterator(expr);

  if symbol_iterator_opt.is_some() {
    return symbol_iterator_opt;
  }

  match expr {
    swc_ecma_ast::Expr::Lit(lit) => match value_from_literal(lit) {
      Ok(value) => Some(value),
      _ => None,
    },
    swc_ecma_ast::Expr::Array(array) => {
      let mut values = Vec::<Value>::new();

      for item in &array.elems {
        values.push(match item {
          Some(item) => {
            if item.spread.is_some() {
              return None;
            }

            static_eval_expr(mc, &item.expr)?
          }
          None => Value::Void,
        });
      }

      Some(Value::Array(Box::new(Array { values })))
    }
    swc_ecma_ast::Expr::Object(object) => {
      let mut properties = Vec::<(Value, Value)>::new();

      for prop in &object.props {
        let (key, value) = match prop {
          swc_ecma_ast::PropOrSpread::Spread(_) => return None,
          swc_ecma_ast::PropOrSpread::Prop(prop) => match &**prop {
            swc_ecma_ast::Prop::Shorthand(_) => return None,
            swc_ecma_ast::Prop::KeyValue(kv) => {
              let key = match &kv.key {
                swc_ecma_ast::PropName::Ident(ident) => Value::String(ident.sym.to_string()),
                swc_ecma_ast::PropName::Str(str) => Value::String(str.value.to_string()),
                swc_ecma_ast::PropName::Num(num) => Value::Number(Number(num.value)),
                swc_ecma_ast::PropName::Computed(computed) => static_eval_expr(mc, &computed.expr)?,
                swc_ecma_ast::PropName::BigInt(bi) => Value::BigInt(bi.value.clone()),
              };

              let value = static_eval_expr(mc, &kv.value)?;

              (key, value)
            }
            swc_ecma_ast::Prop::Assign(_) => return None,
            swc_ecma_ast::Prop::Getter(_) => return None,
            swc_ecma_ast::Prop::Setter(_) => return None,
            swc_ecma_ast::Prop::Method(_) => return None,
          },
        };

        properties.push((key, value));
      }

      Some(Value::Object(Box::new(Object { properties })))
    }
    swc_ecma_ast::Expr::This(_) => None,
    swc_ecma_ast::Expr::Fn(_) => None,
    swc_ecma_ast::Expr::Update(_) => None,
    swc_ecma_ast::Expr::Assign(_) => None,
    swc_ecma_ast::Expr::SuperProp(_) => None,
    swc_ecma_ast::Expr::Call(_) => None,
    swc_ecma_ast::Expr::New(_) => None,
    swc_ecma_ast::Expr::Ident(ident) => match mc
      .scope_analysis
      .lookup(&Ident::from_swc_ident(ident))
      .map(|name| name.value.clone())
    {
      Some(Value::Pointer(p)) => mc.constants_map.get(&p).cloned(),
      Some(value) => Some(value),
      None => None,
    },
    swc_ecma_ast::Expr::TaggedTpl(_) => None,
    swc_ecma_ast::Expr::Arrow(_) => None,
    swc_ecma_ast::Expr::Class(_) => None,
    swc_ecma_ast::Expr::Yield(_) => None,
    swc_ecma_ast::Expr::MetaProp(_) => None,
    swc_ecma_ast::Expr::Await(_) => None,
    swc_ecma_ast::Expr::JSXMember(_) => None,
    swc_ecma_ast::Expr::JSXNamespacedName(_) => None,
    swc_ecma_ast::Expr::JSXEmpty(_) => None,
    swc_ecma_ast::Expr::JSXElement(_) => None,
    swc_ecma_ast::Expr::JSXFragment(_) => None,
    swc_ecma_ast::Expr::TsInstantiation(_) => None,
    swc_ecma_ast::Expr::PrivateName(_) => None,
    swc_ecma_ast::Expr::OptChain(_) => None,
    swc_ecma_ast::Expr::Invalid(_) => None,
    swc_ecma_ast::Expr::Member(_) => None,
    swc_ecma_ast::Expr::Cond(_) => None,
    swc_ecma_ast::Expr::Unary(unary) => match unary.op {
      swc_ecma_ast::UnaryOp::Minus => match static_eval_expr(mc, &unary.arg)? {
        Value::Number(Number(x)) => Some(Value::Number(Number(-x))),
        Value::BigInt(bi) => Some(Value::BigInt(-bi)),
        _ => None,
      },
      swc_ecma_ast::UnaryOp::Plus => match static_eval_expr(mc, &unary.arg)? {
        Value::Number(Number(x)) => Some(Value::Number(Number(x))),
        Value::BigInt(bi) => Some(Value::BigInt(bi)),
        _ => None,
      },
      swc_ecma_ast::UnaryOp::Bang => None,
      swc_ecma_ast::UnaryOp::Tilde => match static_eval_expr(mc, &unary.arg)? {
        Value::Number(Number(x)) => Some(Value::Number(Number(!to_i32(x) as f64))),
        Value::BigInt(bi) => Some(Value::BigInt(!bi)),
        _ => None,
      },
      swc_ecma_ast::UnaryOp::TypeOf => None,
      swc_ecma_ast::UnaryOp::Void => None,
      swc_ecma_ast::UnaryOp::Delete => None,
    },
    swc_ecma_ast::Expr::Bin(_) => None,
    swc_ecma_ast::Expr::Seq(seq) => {
      let mut last = Value::Void;

      for expr in &seq.exprs {
        last = static_eval_expr(mc, expr)?;
      }

      Some(last)
    }
    swc_ecma_ast::Expr::Tpl(tpl) => 'b: {
      let len = tpl.exprs.len();
      assert_eq!(tpl.quasis.len(), len + 1);

      if len == 0 {
        break 'b Some(Value::String(tpl.quasis[0].raw.to_string()));
      }

      None // TODO
    }
    swc_ecma_ast::Expr::Paren(paren) => static_eval_expr(mc, &paren.expr),
    swc_ecma_ast::Expr::TsTypeAssertion(tta) => static_eval_expr(mc, &tta.expr),
    swc_ecma_ast::Expr::TsConstAssertion(tca) => static_eval_expr(mc, &tca.expr),
    swc_ecma_ast::Expr::TsNonNull(tnn) => static_eval_expr(mc, &tnn.expr),
    swc_ecma_ast::Expr::TsAs(ta) => static_eval_expr(mc, &ta.expr),
  }
}

fn as_symbol_iterator(expr: &swc_ecma_ast::Expr) -> Option<Value> {
  let member_expr = match expr {
    swc_ecma_ast::Expr::Member(member_expr) => member_expr,
    _ => return None,
  };

  match &*member_expr.obj {
    swc_ecma_ast::Expr::Ident(ident) => {
      if ident.sym.to_string() != "Symbol" {
        return None;
      }
    }
    _ => return None,
  };

  match &member_expr.prop {
    swc_ecma_ast::MemberProp::Ident(ident) => {
      if ident.sym.to_string() != "iterator" {
        return None;
      }
    }
    _ => return None,
  }

  Some(Value::Builtin(Builtin {
    name: "SymbolIterator".to_string(),
  }))
}
