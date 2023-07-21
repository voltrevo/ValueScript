#[derive(Debug)]
pub struct Ident {
  pub sym: swc_atoms::JsWord,
  pub span: swc_common::Span,
}

impl Ident {
  pub fn from_swc_ident(ident: &swc_ecma_ast::Ident) -> Self {
    Ident {
      sym: ident.sym.clone(),
      span: ident.span,
    }
  }

  pub fn this(span: swc_common::Span) -> Self {
    Ident {
      sym: swc_atoms::JsWord::from("this"),
      span,
    }
  }
}
