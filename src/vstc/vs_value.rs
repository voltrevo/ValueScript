use std::rc::Rc;

type Val = Rc<dyn VsValue>;

#[derive(PartialEq)]
enum VsType {
  Undefined,
  Null,
  Bool,
  Number,
  String,
  Array,
  Object,
  Function,
}

impl VsType {
  fn as_val(&self) -> Val {
    return VsString::from_str(match self {
      Undefined => "undefined",
      Null => "object",
      Bool => "boolean",
      Number => "number",
      String => "string",
      Array => "object",
      Object => "object",
      Function => "function",
    });
  }
}

trait VsValue {
  fn typeof_(&self) -> VsType;
  fn to_string(&self) -> String;
  fn to_number(&self) -> f64;
}

pub struct VsNumber {
  value: f64,
}

impl VsNumber {
  fn from_f64(value: f64) -> Val {
    return Rc::new(VsNumber { value: value });
  }
}

pub struct VsString {
  value: String,
}

impl VsString {
  fn from_str(value: &str) -> Val {
    return Rc::new(VsString { value: value.to_string() });
  }

  fn from_string(value: String) -> Val {
    return Rc::new(VsString { value: value });
  }
}

impl VsValue for VsNumber {
  fn typeof_(&self) -> VsType {
    return VsType::Number;
  }

  fn to_string(&self) -> String {
    return self.value.to_string();
  }

  fn to_number(&self) -> f64 {
    return self.value;
  }
}

impl VsValue for VsString {
  fn typeof_(&self) -> VsType {
    return VsType::String;
  }

  fn to_string(&self) -> String {
    return self.value.clone();
  }

  fn to_number(&self) -> f64 {
    std::panic!("not implemented");
  }
}

fn add(left: &Rc<dyn VsValue>, right: &Rc<dyn VsValue>) -> Rc<dyn VsValue> {
  if left.typeof_() == VsType::String || right.typeof_() == VsType::String {
    // TODO: Do we need .clone() on the left here? If not, why?
    return VsString::from_string(left.to_string() + &right.to_string());
  }

  return VsNumber::from_f64(left.to_number() + right.to_number());
}
