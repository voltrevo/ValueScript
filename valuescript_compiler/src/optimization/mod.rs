mod collapse_pointers_of_pointers;
mod extract_constants;
mod optimize;
mod remove_meta_lines;
mod remove_noops;
mod shake_tree;
mod simplify;
pub mod try_to_val;
pub mod try_to_value;

pub use optimize::optimize;
