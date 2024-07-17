use std::cell::RefCell;

use swc_common::Spanned;
use valuescript_common::to_i32;

use crate::{
  asm::{Array, Builtin, Number, Object, Value},
  diagnostic::{DiagnosticContainer, DiagnosticReporter},
  expression_compiler::value_from_literal,
  function_compiler::Functionish,
  ident::Ident,
  module_compiler::ModuleCompiler,
  Diagnostic,
};

pub struct StaticExpressionCompiler<'a> {
  pub mc: &'a mut ModuleCompiler,
}

impl<'a> DiagnosticContainer for StaticExpressionCompiler<'a> {
  fn diagnostics_mut(&self) -> &RefCell<Vec<Diagnostic>> {
    self.mc.diagnostics_mut()
  }
}

impl<'a> StaticExpressionCompiler<'a> {
  pub fn new(mc: &'a mut ModuleCompiler) -> Self {
    StaticExpressionCompiler { mc }
  }

  pub fn expr(&mut self, expr: &swc_ecma_ast::Expr) -> Value {
    let symbol_iterator_opt = as_symbol_iterator(expr);

    if let Some(symbol_iterator) = symbol_iterator_opt {
      return symbol_iterator;
    }

    match expr {
      swc_ecma_ast::Expr::Lit(lit) => match value_from_literal(lit) {
        Ok(value) => value,
        Err(msg) => {
          self.internal_error(expr.span(), &format!("Failed to compile literal: {}", msg));
          Value::String("(error)".to_string())
        }
      },
      swc_ecma_ast::Expr::Array(array) => {
        let mut values = Vec::<Value>::new();

        for item in &array.elems {
          values.push(match item {
            Some(item) => {
              if item.spread.is_some() {
                self.todo(expr.span(), "item.spread in static expression");
                return Value::String("(error)".to_string());
              }

              self.expr(&item.expr)
            }
            None => Value::Void,
          });
        }

        Value::Array(Box::new(Array { values }))
      }
      swc_ecma_ast::Expr::Object(object) => {
        let mut properties = Vec::<(Value, Value)>::new();

        for prop in &object.props {
          let (key, value) = match prop {
            swc_ecma_ast::PropOrSpread::Spread(_) => {
              self.todo(prop.span(), "Static spread");
              return Value::String("(error)".to_string());
            }
            swc_ecma_ast::PropOrSpread::Prop(prop) => match &**prop {
              swc_ecma_ast::Prop::Shorthand(_) => {
                self.todo(prop.span(), "Static object shorthand");
                return Value::String("(error)".to_string());
              }
              swc_ecma_ast::Prop::KeyValue(kv) => {
                let key = match &kv.key {
                  swc_ecma_ast::PropName::Ident(ident) => Value::String(ident.sym.to_string()),
                  swc_ecma_ast::PropName::Str(str) => Value::String(str.value.to_string()),
                  swc_ecma_ast::PropName::Num(num) => Value::Number(Number(num.value)),
                  swc_ecma_ast::PropName::Computed(computed) => self.expr(&computed.expr),
                  swc_ecma_ast::PropName::BigInt(bi) => Value::BigInt(bi.value.clone()),
                };

                let value = self.expr(&kv.value);

                (key, value)
              }
              swc_ecma_ast::Prop::Assign(_)
              | swc_ecma_ast::Prop::Getter(_)
              | swc_ecma_ast::Prop::Setter(_) => {
                self.todo(prop.span(), "This type of static prop");
                return Value::String("(error)".to_string());
              }
              swc_ecma_ast::Prop::Method(method) => {
                let key = self.prop_name(&method.key);

                let fn_ident = match &method.key {
                  swc_ecma_ast::PropName::Ident(ident) => Some(ident.clone()),
                  _ => None,
                };

                let fn_name = fn_ident.clone().map(|ident| ident.sym.to_string());

                let p = match &fn_name {
                  Some(name) => self.mc.allocate_defn(name),
                  None => self.mc.allocate_defn_numbered("_anon"),
                };

                self.mc.compile_fn(
                  p.clone(),
                  Functionish::Fn(fn_ident, method.function.clone()),
                );

                (key, Value::Pointer(p))
              }
            },
          };

          properties.push((key, value));
        }

        Value::Object(Box::new(Object { properties }))
      }
      swc_ecma_ast::Expr::This(_)
      | swc_ecma_ast::Expr::Update(_)
      | swc_ecma_ast::Expr::Assign(_)
      | swc_ecma_ast::Expr::SuperProp(_)
      | swc_ecma_ast::Expr::Call(_)
      | swc_ecma_ast::Expr::New(_) => {
        self.todo(expr.span(), "This type of static expr");
        Value::String("(error)".to_string())
      }
      swc_ecma_ast::Expr::Ident(ident) => match self
        .mc
        .scope_analysis
        .lookup(&Ident::from_swc_ident(ident))
        .map(|name| name.value.clone())
      {
        Some(Value::Pointer(p)) => self
          .mc
          .constants_map
          .get(&p)
          .cloned()
          .unwrap_or_else(|| Value::Pointer(p)),
        Some(value) => value,
        None => {
          self.internal_error(ident.span, "Identifier not found");
          Value::String("(error)".to_string())
        }
      },
      swc_ecma_ast::Expr::Fn(fn_) => {
        let fn_name = fn_.ident.clone().map(|ident| ident.sym.to_string());

        let p = match &fn_name {
          Some(name) => self.mc.allocate_defn(name),
          None => self.mc.allocate_defn_numbered("_anon"),
        };

        self.mc.compile_fn(
          p.clone(),
          Functionish::Fn(fn_.ident.clone(), fn_.function.clone()),
        );

        Value::Pointer(p)
      }
      swc_ecma_ast::Expr::Arrow(arrow) => {
        let p = self.mc.allocate_defn_numbered("_anon");

        self
          .mc
          .compile_fn(p.clone(), Functionish::Arrow(arrow.clone()));

        Value::Pointer(p)
      }
      swc_ecma_ast::Expr::Class(class) => Value::Pointer(self.mc.compile_class(
        None,
        class.ident.as_ref(),
        &class.class,
      )),
      swc_ecma_ast::Expr::TaggedTpl(_)
      | swc_ecma_ast::Expr::Yield(_)
      | swc_ecma_ast::Expr::MetaProp(_)
      | swc_ecma_ast::Expr::Await(_)
      | swc_ecma_ast::Expr::JSXMember(_)
      | swc_ecma_ast::Expr::JSXNamespacedName(_)
      | swc_ecma_ast::Expr::JSXEmpty(_)
      | swc_ecma_ast::Expr::JSXElement(_)
      | swc_ecma_ast::Expr::JSXFragment(_)
      | swc_ecma_ast::Expr::TsInstantiation(_)
      | swc_ecma_ast::Expr::PrivateName(_)
      | swc_ecma_ast::Expr::OptChain(_)
      | swc_ecma_ast::Expr::Invalid(_)
      | swc_ecma_ast::Expr::Member(_)
      | swc_ecma_ast::Expr::Cond(_) => {
        self.todo(expr.span(), "This type of static expr");
        Value::String("(error)".to_string())
      }
      swc_ecma_ast::Expr::Unary(unary) => match unary.op {
        swc_ecma_ast::UnaryOp::Minus => match self.expr(&unary.arg) {
          Value::Number(Number(x)) => Value::Number(Number(-x)),
          Value::BigInt(bi) => Value::BigInt(-bi),
          _ => {
            self.todo(unary.span, "Static eval for this case");
            Value::String("(error)".to_string())
          }
        },
        swc_ecma_ast::UnaryOp::Plus => match self.expr(&unary.arg) {
          Value::Number(Number(x)) => Value::Number(Number(x)),
          Value::BigInt(bi) => Value::BigInt(bi),
          _ => {
            self.todo(unary.span, "Static eval for this case");
            Value::String("(error)".to_string())
          }
        },
        swc_ecma_ast::UnaryOp::Bang => {
          self.todo(expr.span(), "Static eval of ! operator");
          Value::String("(error)".to_string())
        }
        swc_ecma_ast::UnaryOp::Tilde => match self.expr(&unary.arg) {
          Value::Number(Number(x)) => Value::Number(Number(!to_i32(x) as f64)),
          Value::BigInt(bi) => Value::BigInt(!bi),
          _ => {
            self.todo(unary.span, "Static eval for this case");
            Value::String("(error)".to_string())
          }
        },
        swc_ecma_ast::UnaryOp::TypeOf
        | swc_ecma_ast::UnaryOp::Void
        | swc_ecma_ast::UnaryOp::Delete => {
          self.todo(unary.span, "Static eval for this case");
          Value::String("(error)".to_string())
        }
      },
      swc_ecma_ast::Expr::Bin(_) => {
        self.todo(expr.span(), "Static eval of binary operator");
        Value::String("(error)".to_string())
      }
      swc_ecma_ast::Expr::Seq(seq) => {
        let mut last = Value::Void;

        for expr in &seq.exprs {
          last = self.expr(expr);
        }

        last
      }
      swc_ecma_ast::Expr::Tpl(tpl) => 'b: {
        let len = tpl.exprs.len();
        assert_eq!(tpl.quasis.len(), len + 1);

        if len == 0 {
          break 'b Value::String(tpl.quasis[0].raw.to_string());
        }

        self.todo(tpl.span, "Static eval of template literal");
        Value::String("(error)".to_string())
      }
      swc_ecma_ast::Expr::Paren(paren) => self.expr(&paren.expr),
      swc_ecma_ast::Expr::TsTypeAssertion(tta) => self.expr(&tta.expr),
      swc_ecma_ast::Expr::TsConstAssertion(tca) => self.expr(&tca.expr),
      swc_ecma_ast::Expr::TsNonNull(tnn) => self.expr(&tnn.expr),
      swc_ecma_ast::Expr::TsAs(ta) => self.expr(&ta.expr),
    }
  }

  pub fn prop_name(&mut self, prop_name: &swc_ecma_ast::PropName) -> Value {
    match prop_name {
      swc_ecma_ast::PropName::Ident(ident) => Value::String(ident.sym.to_string()),
      swc_ecma_ast::PropName::Str(str) => Value::String(str.value.to_string()),
      swc_ecma_ast::PropName::Num(num) => Value::String(num.value.to_string()),
      swc_ecma_ast::PropName::Computed(computed) => self.expr(&computed.expr),
      swc_ecma_ast::PropName::BigInt(bi) => Value::String(bi.value.to_string()),
    }
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
