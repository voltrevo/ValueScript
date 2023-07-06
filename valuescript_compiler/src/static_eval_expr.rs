use crate::{
  asm::{Array, Builtin, Number, Object, Value},
  expression_compiler::value_from_literal,
};

pub fn static_eval_expr(expr: &swc_ecma_ast::Expr) -> Option<Value> {
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

            static_eval_expr(&item.expr)?
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
                swc_ecma_ast::PropName::Computed(computed) => static_eval_expr(&computed.expr)?,
                swc_ecma_ast::PropName::BigInt(bi) => Value::BigInt(bi.value.clone()),
              };

              let value = static_eval_expr(&kv.value)?;

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
    _ => None,
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

  return Some(Value::Builtin(Builtin {
    name: "SymbolIterator".to_string(),
  }));
}
