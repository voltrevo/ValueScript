use num_derive::{FromPrimitive, ToPrimitive};

use crate::vs_value::{ToVal, Val};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive)]
pub enum VsSymbol {
  ITERATOR,
}

pub fn symbol_to_name(symbol: VsSymbol) -> &'static str {
  match symbol {
    VsSymbol::ITERATOR => "iterator",
  }
}

impl ToVal for VsSymbol {
  fn to_val(self) -> Val {
    Val::Symbol(self)
  }
}
