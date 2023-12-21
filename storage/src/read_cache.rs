use std::{any::Any, collections::HashMap};

pub type ReadCache = HashMap<(u64, u64, u64), Box<dyn Any>>;
