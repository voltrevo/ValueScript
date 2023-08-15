use swc_common::BytePos;
use tiny_keccak::{Hasher, Keccak};

use crate::asm::Hash;

pub fn src_hash(source: &str, span: swc_common::Span) -> Hash {
  let BytePos(start) = span.lo;
  let BytePos(end) = span.hi;

  let chars = source
    .chars()
    .skip(start as usize)
    .take((end - start) as usize);

  let mut k = Keccak::v256();
  k.update(chars.collect::<String>().as_bytes());

  let mut output = [0u8; 32];
  k.finalize(&mut output);

  Hash(output)
}
