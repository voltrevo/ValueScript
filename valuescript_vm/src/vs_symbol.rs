#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VsSymbol {
  ITERATOR,
}

pub fn symbol_to_name(symbol: VsSymbol) -> &'static str {
  match symbol {
    VsSymbol::ITERATOR => "iterator",
  }
}
