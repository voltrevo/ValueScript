use std::rc::Rc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize_rc<T: Serialize, S: Serializer>(
  data: &Rc<T>,
  serializer: S,
) -> Result<S::Ok, S::Error> {
  data.serialize(serializer)
}

pub fn deserialize_rc<'de, T: Deserialize<'de>, D: Deserializer<'de>>(
  deserializer: D,
) -> Result<Rc<T>, D::Error> {
  let data = T::deserialize(deserializer)?;
  Ok(Rc::new(data))
}
