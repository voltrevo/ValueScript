use std::rc::Rc;

use crate::builtins::type_error_builtin::ToTypeError;
use crate::native_frame_function::NativeFrameFunction;
use crate::native_function::ThisWrapper;
use crate::stack_frame::FrameStepResult;
use crate::stack_frame::{CallResult, FrameStepOk, StackFrameTrait};
use crate::vs_array::VsArray;
use crate::vs_value::{LoadFunctionResult, ToVal, Val, ValTrait};

pub static SORT: NativeFrameFunction = NativeFrameFunction {
  make_frame: || {
    Box::new(SortFrame {
      this: None,
      comparator: Val::Void,
      param_i: 0,
      tree: SortTreeNode {
        data: SortTreeNodeData::Sorted(vec![]),
      },
      started: false,
    })
  },
};

struct SortFrame {
  this: Option<Rc<VsArray>>,

  comparator: Val,
  param_i: usize,

  tree: SortTreeNode,
  started: bool,
}

struct VecPos<T> {
  vec: Vec<T>,
  pos: usize,
}

struct VecSlice<'a, T> {
  vec: &'a Vec<T>,
  start: usize,
  end: usize,
}

struct SortTreeNode {
  data: SortTreeNodeData,
}

impl SortTreeNode {
  fn new(vals: VecSlice<Val>) -> SortTreeNode {
    let len = vals.end - vals.start;

    if len <= 1 {
      let mut sorted = vec![];

      for i in vals.start..vals.end {
        sorted.push(vals.vec[i].clone());
      }

      return SortTreeNode {
        data: SortTreeNodeData::Sorted(sorted),
      };
    }

    if len == 2 {
      return SortTreeNode {
        data: SortTreeNodeData::Sorting(
          Vec::new(),
          VecPos {
            vec: vec![vals.vec[vals.start].clone()],
            pos: 0,
          },
          VecPos {
            vec: vec![vals.vec[vals.start + 1].clone()],
            pos: 0,
          },
        ),
      };
    }

    let mid = vals.start + (vals.end - vals.start) / 2;

    return SortTreeNode {
      data: SortTreeNodeData::Branch(
        Box::new(SortTreeNode::new(VecSlice {
          vec: vals.vec,
          start: vals.start,
          end: mid,
        })),
        Box::new(SortTreeNode::new(VecSlice {
          vec: vals.vec,
          start: mid,
          end: vals.end,
        })),
      ),
    };
  }

  fn get_compare_elements(&self) -> Option<(Val, Val)> {
    match &self.data {
      SortTreeNodeData::Branch(left, right) => {
        return left
          .get_compare_elements()
          .or_else(|| right.get_compare_elements());
      }
      SortTreeNodeData::Sorting(_vals, left, right) => {
        let lval_opt = left.vec.get(left.pos);
        let rval_opt = right.vec.get(right.pos);

        match (lval_opt, rval_opt) {
          (Some(lval), Some(rval)) => {
            return Some((lval.clone(), rval.clone()));
          }
          _ => {
            panic!("Failed to get compare elements from sorting state");
          }
        }
      }
      SortTreeNodeData::Sorted(_) => {
        return None;
      }
    };
  }

  fn apply_outcome(&mut self, should_swap: bool) {
    match &mut self.data {
      SortTreeNodeData::Branch(left, right) => {
        match &mut left.data {
          SortTreeNodeData::Branch(_, _) | SortTreeNodeData::Sorting(_, _, _) => {
            left.apply_outcome(should_swap);
          }
          SortTreeNodeData::Sorted(left_vals) => {
            right.apply_outcome(should_swap);

            match &mut right.data {
              SortTreeNodeData::Sorted(right_vals) => {
                let mut owned_left_vals = vec![];
                std::mem::swap(&mut owned_left_vals, left_vals);
                let mut owned_right_vals = vec![];
                std::mem::swap(&mut owned_right_vals, right_vals);

                self.data = SortTreeNodeData::Sorting(
                  vec![],
                  VecPos {
                    vec: owned_left_vals,
                    pos: 0,
                  },
                  VecPos {
                    vec: owned_right_vals,
                    pos: 0,
                  },
                );
              }
              _ => {}
            };
          }
        };
      }

      SortTreeNodeData::Sorted(_) => {
        panic!("Failed to apply outcome");
      }

      SortTreeNodeData::Sorting(vals, left, right) => {
        let lval_opt = left.vec.get(left.pos);
        let rval_opt = right.vec.get(right.pos);

        match (lval_opt, rval_opt) {
          (Some(lval), Some(rval)) => match should_swap {
            false => {
              vals.push(lval.clone());
              left.pos += 1;
            }
            true => {
              vals.push(rval.clone());
              right.pos += 1;
            }
          },
          _ => panic!("Failed to apply outcome"),
        };

        if left.pos == left.vec.len() || right.pos == right.vec.len() {
          for i in left.pos..left.vec.len() {
            vals.push(left.vec[i].clone());
          }

          for i in right.pos..right.vec.len() {
            vals.push(right.vec[i].clone());
          }

          let mut owned_vals = vec![];
          std::mem::swap(&mut owned_vals, vals);

          self.data = SortTreeNodeData::Sorted(owned_vals);
        }
      }
    };
  }
}

enum SortTreeNodeData {
  Branch(Box<SortTreeNode>, Box<SortTreeNode>),
  Sorting(Vec<Val>, VecPos<Val>, VecPos<Val>),
  Sorted(Vec<Val>),
}

impl StackFrameTrait for SortFrame {
  fn write_this(&mut self, const_: bool, this: Val) -> Result<(), Val> {
    if const_ {
      return Err("Cannot sort const array".to_type_error());
    }

    self.this = this.as_array_data();
    Ok(())
  }

  fn write_param(&mut self, param: Val) {
    match self.param_i {
      0 => {
        self.comparator = param;
      }
      _ => {}
    };

    self.param_i += 1;
  }

  fn step(&mut self) -> FrameStepResult {
    if !self.started {
      let array_data = match &mut self.this {
        None => return Err("array fn called on non-array".to_type_error()),
        Some(ad) => ad,
      };

      match self.comparator {
        Val::Void => {
          let array_data_mut = Rc::make_mut(array_data);

          array_data_mut
            .elements
            .sort_by(|a, b| a.val_to_string().cmp(&b.val_to_string()));

          return Ok(FrameStepOk::Pop(CallResult {
            return_: Val::Array(array_data.clone()),
            this: Val::Array(array_data.clone()),
          }));
        }
        _ => {
          self.tree = SortTreeNode::new(VecSlice {
            vec: &array_data.elements,
            start: 0,
            end: array_data.elements.len(),
          });

          self.started = true;
        }
      };
    }

    Ok(match self.tree.get_compare_elements() {
      None => match &mut self.tree.data {
        SortTreeNodeData::Sorted(vals) => {
          let mut owned_vals = vec![];
          std::mem::swap(&mut owned_vals, vals);
          let res = owned_vals.to_val();

          FrameStepOk::Pop(CallResult {
            return_: res.clone(),
            this: res,
          })
        }
        _ => panic!("This shouldn't happen"), // TODO: Internal errors
      },
      Some((left, right)) => match self.comparator.load_function() {
        LoadFunctionResult::NotAFunction => {
          return Err("comparator is not a function".to_type_error());
        }
        LoadFunctionResult::NativeFunction(native_fn) => {
          let res = native_fn(
            ThisWrapper::new(true, &mut Val::Undefined),
            vec![left, right],
          )?
          .to_number();

          let should_swap = match res.is_nan() {
            true => false,
            false => res > 0_f64,
          };

          self.tree.apply_outcome(should_swap);
          FrameStepOk::Continue
        }
        LoadFunctionResult::StackFrame(mut new_frame) => {
          new_frame.write_param(left);
          new_frame.write_param(right);
          FrameStepOk::Push(new_frame)
        }
      },
    })
  }

  fn apply_call_result(&mut self, call_result: CallResult) {
    let res = call_result.return_.to_number();

    let should_swap = match res.is_nan() {
      true => false,
      false => res > 0_f64,
    };

    self.tree.apply_outcome(should_swap);
  }

  fn get_call_result(&mut self) -> CallResult {
    panic!("Not appropriate for SortFrame")
  }

  fn catch_exception(&mut self, _exception: Val) -> bool {
    return false;
  }
}
