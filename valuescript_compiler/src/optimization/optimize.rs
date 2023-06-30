use crate::asm::Module;
use crate::name_allocator::NameAllocator;

use super::collapse_pointers_of_pointers::collapse_pointers_of_pointers;
use super::extract_constants::extract_constants;
use super::remove_meta_lines::remove_meta_lines;
use super::remove_noops::remove_noops;
use super::shake_tree::shake_tree;
use super::simplify::simplify;

pub fn optimize(module: &mut Module, pointer_allocator: &mut NameAllocator) {
  collapse_pointers_of_pointers(module);
  extract_constants(module, pointer_allocator);
  shake_tree(module);
  simplify(module);
  remove_noops(module);
  remove_meta_lines(module);
}
