use std::{fmt, rc::Rc};

use num_bigint::BigInt;

use crate::{
  builtins::type_error_builtin::ToTypeError,
  vs_array::VsArray,
  vs_class::VsClass,
  vs_value::{Val, VsType},
  LoadFunctionResult, ValTrait,
};

#[derive(Clone)]
pub struct JsxElement {
  pub tag: Option<String>,
  pub attrs: Vec<(String, Val)>,
  pub children: Vec<Val>,
}

impl ValTrait for JsxElement {
  fn typeof_(&self) -> VsType {
    VsType::Object
  }

  fn to_number(&self) -> f64 {
    f64::NAN
  }

  fn to_index(&self) -> Option<usize> {
    None
  }

  fn is_primitive(&self) -> bool {
    false
  }

  fn is_truthy(&self) -> bool {
    true
  }

  fn is_nullish(&self) -> bool {
    false
  }

  fn bind(&self, _params: Vec<Val>) -> Option<Val> {
    None
  }

  fn as_bigint_data(&self) -> Option<BigInt> {
    None
  }

  fn as_array_data(&self) -> Option<Rc<VsArray>> {
    None
  }

  fn as_class_data(&self) -> Option<Rc<VsClass>> {
    None
  }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn sub(&self, _key: &Val) -> Result<Val, Val> {
    Ok(Val::Undefined)
  }

  fn has(&self, _key: &Val) -> Option<bool> {
    Some(false)
  }

  fn submov(&mut self, _key: &Val, _value: Val) -> Result<(), Val> {
    Err("Cannot assign to subscript of jsx element".to_type_error())
  }

  fn pretty_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.tag.is_none() && self.children.is_empty() {
      return write!(f, "\x1b[36m<></>\x1b[39m");
    }

    let tag_str = match &self.tag {
      Some(str) => str.clone(),
      None => "".to_owned(),
    };

    if self.children.is_empty() {
      write!(f, "\x1b[36m<{}\x1b[39m", tag_str)?;
      write_attributes_pretty(f, &self.attrs)?;
      write!(f, " \x1b[36m/>\x1b[39m")
    } else {
      write!(f, "\x1b[36m<{}\x1b[39m", tag_str)?;
      write_attributes_pretty(f, &self.attrs)?;
      write!(f, "\x1b[36m>\x1b[39m")?;

      for child in &self.children {
        if is_jsx_element(child) {
          write!(f, "{}", child.pretty())?;
        } else {
          write!(f, "{}", child)?;
        }
      }

      write!(f, "\x1b[36m</{}>\x1b[39m", tag_str)
    }
  }

  fn codify(&self) -> String {
    self.to_string()
  }
}

impl fmt::Display for JsxElement {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let tag_str = match &self.tag {
      Some(str) => str.clone(),
      None => "".to_owned(),
    };

    if self.children.is_empty() {
      write!(f, "<{}", tag_str)?;
      write_attributes(f, &self.attrs)?;
      write!(f, " />")
    } else {
      write!(f, "<{}", tag_str)?;
      write_attributes(f, &self.attrs)?;
      write!(f, ">")?;

      for child in &self.children {
        write!(f, "{}", child)?;
      }

      write!(f, "</{}>", tag_str)
    }
  }
}

fn write_attributes(f: &mut fmt::Formatter<'_>, attrs: &Vec<(String, Val)>) -> fmt::Result {
  for (key, val) in attrs {
    write!(f, " {}=\"{}\"", key, val)?;
  }

  Ok(())
}

fn write_attributes_pretty(f: &mut fmt::Formatter<'_>, attrs: &Vec<(String, Val)>) -> fmt::Result {
  for (key, val) in attrs {
    write!(f, " {}=\x1b[33m\"{}\"\x1b[39m", key, val)?;
  }

  Ok(())
}

pub fn is_jsx_element(val: &Val) -> bool {
  match val {
    Val::Dynamic(dynamic) => dynamic.as_any().is::<JsxElement>(),
    Val::StoragePtr(ptr) => is_jsx_element(&ptr.get()),
    _ => false,
  }
}
