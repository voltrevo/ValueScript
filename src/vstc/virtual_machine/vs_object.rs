use std::collections::BTreeMap;

use super::vs_value::Val;

#[derive(Clone)]
pub struct VsObject {
  pub string_map: BTreeMap<String, Val>,
  pub prototype: Option<Val>,
}
