use std::rc::Rc;

use serde::{Deserializer, Serializer};

pub fn serialize_rc<T, S>(data: &Rc<T>, serializer: S) -> Result<S::Ok, S::Error>
where
  T: serde::Serialize,
  S: Serializer,
{
  data.serialize(serializer)
}

pub fn deserialize_rc<'de, T, D>(deserializer: D) -> Result<Rc<T>, D::Error>
where
  T: serde::Deserialize<'de>,
  D: Deserializer<'de>,
{
  let data = T::deserialize(deserializer)?;
  Ok(Rc::new(data))
}
