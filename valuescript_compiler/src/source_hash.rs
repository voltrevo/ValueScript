use swc_common::BytePos;
use tiny_keccak::{Hasher, Keccak};

#[allow(dead_code)]
pub fn source_hash(source: &str, span: swc_common::Span) -> [u8; 32] {
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

  output
}
