use crate::asm::Module;
use crate::name_allocator::NameAllocator;

use super::extract_constants::extract_constants;
use super::reduce_instructions::reduce_instructions;
use super::remove_meta_lines::remove_meta_lines;
use super::remove_unused_labels::remove_unused_labels;
use super::remove_unused_registers::remove_unused_registers;
use super::shake_tree::shake_tree;
use super::simplify::simplify;
use super::simplify_jumps::simplify_jumps;

pub fn optimize(module: &mut Module, pointer_allocator: &mut NameAllocator) {
  shake_tree(module);

  let passes = 3;

  for i in 0..passes {
    simplify(module, i == passes - 1);
    reduce_instructions(module);
    remove_unused_labels(module);
    remove_unused_registers(module);
    reduce_instructions(module);
    simplify_jumps(module);
  }

  remove_meta_lines(module);
  extract_constants(module, pointer_allocator);

  // After possibly repeated optimization, this ensures that the pointers are ordered correctly.
  // TODO: Consider a dedicated step that's only about pointer ordering and not tree shaking.
  shake_tree(module);
}
