use std::process::exit;

use valuescript_vm::{
  operations::{op_less, op_minus, op_plus},
  vs_value::{ToVal, Val},
  ValTrait, Vallish,
};

pub fn main() {
  let result = fib(35.0.to_val());
  // let result: Result<Val, Val> = Ok(fib2(35.0).to_val());

  match result {
    Ok(result) => println!("{}", result.pretty()),
    Err(err) => {
      println!("Uncaught exception: {}", err.pretty());
      exit(1);
    }
  }
}

// Via bytecode: 12.6s

// 1.52s
// pub fn fib(n: Val) -> Result<Val, Val> {
//   let mut _return = Val::Undefined;

//   let mut _tmp0 = op_less(n.clone(), 2.0.to_val())?;
//   _tmp0 = op_not(_tmp0.clone());

//   if !_tmp0.is_truthy() {
//     _return = n.clone();
//     return Ok(_return);
//   }

//   let mut _tmp1 = op_minus(n.clone(), 1.0.to_val())?;
//   let mut _tmp2 = fib(_tmp1.clone())?;
//   _tmp1 = op_minus(n.clone(), 2.0.to_val())?;
//   let mut _tmp3 = fib(_tmp1.clone())?;
//   _return = op_plus(_tmp2.clone(), _tmp3.clone())?;

//   return Ok(_return);
// }

// 1.17s
// pub fn fib(n: Val) -> Result<Val, Val> {
//   let mut _return = Val::Undefined;

//   let mut _tmp0 = op_less(n.clone(), 2.0.to_val())?;
//   _tmp0 = op_not(_tmp0);

//   if !_tmp0.is_truthy() {
//     _return = n;
//     return Ok(_return);
//   }

//   let mut _tmp1 = op_minus(n.clone(), 1.0.to_val())?;
//   let mut _tmp2 = fib(_tmp1)?;
//   _tmp1 = op_minus(n, 2.0.to_val())?;
//   let mut _tmp3 = fib(_tmp1)?;
//   _return = op_plus(_tmp2, _tmp3)?;

//   return Ok(_return);
// }

// 1.09s
// Update: 1.06s
pub fn fib(n: Val) -> Result<Val, Val> {
  let mut _return = Val::Undefined;

  let mut _tmp0 = op_less(Vallish::Ref(&n), Vallish::Own(2.0.to_val()))?;

  if _tmp0.is_truthy() {
    _return = n;
    return Ok(_return);
  }

  let mut _tmp1 = op_minus(Vallish::Ref(&n), Vallish::Own(1.0.to_val()))?;
  let mut _tmp2 = fib(_tmp1)?;
  _tmp1 = op_minus(Vallish::Own(n), Vallish::Own(2.0.to_val()))?;
  let mut _tmp3 = fib(_tmp1)?;
  _return = op_plus(Vallish::Own(_tmp2), Vallish::Own(_tmp3))?;

  return Ok(_return);
}

// 0.120s
pub fn fib2(n: f64) -> f64 {
  if n < 2.0 {
    return n;
  }

  return fib2(n - 1.0) + fib2(n - 2.0);
}
