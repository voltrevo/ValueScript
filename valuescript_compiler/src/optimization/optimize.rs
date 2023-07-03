use crate::asm::Module;
use crate::name_allocator::NameAllocator;

use super::collapse_pointers_of_pointers::collapse_pointers_of_pointers;
use super::extract_constants::extract_constants;
use super::reduce_instructions::reduce_instructions;
use super::remove_meta_lines::remove_meta_lines;
use super::remove_unused_labels::remove_unused_labels;
use super::shake_tree::shake_tree;
use super::simplify::simplify;

pub fn optimize(module: &mut Module, pointer_allocator: &mut NameAllocator) {
  collapse_pointers_of_pointers(module);
  shake_tree(module);

  for _ in 0..2 {
    simplify(module);
    reduce_instructions(module);
    remove_unused_labels(module);
  }

  remove_meta_lines(module);
  extract_constants(module, pointer_allocator);

  // After possibly repeated optimization, this ensures that the pointers are ordered correctly.
  // TODO: Consider a dedicated step that's only about pointer ordering and not tree shaking.
  shake_tree(module);
}
