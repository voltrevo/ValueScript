use crate::vs_value::Val;

pub enum Vallish<'a> {
  Own(Val),
  Ref(&'a Val),
}

impl<'a> Vallish<'a> {
  pub fn get_own(self) -> Val {
    match self {
      Vallish::Own(val) => val,
      Vallish::Ref(val) => val.clone(),
    }
  }

  pub fn get_ref(&self) -> &Val {
    match self {
      Vallish::Own(val) => val,
      Vallish::Ref(val) => val,
    }
  }
}
