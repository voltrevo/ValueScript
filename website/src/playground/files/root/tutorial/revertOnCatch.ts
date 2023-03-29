// We also deviate from JavaScript in a few other ways:
// 
// - Try blocks are transactional

export default function () {
  let x = 0;

  try {
    x++;
    throw new Error("boom");
  } catch {}

  return x;
  // JavaScript:  1
  // ValueScript: 0
}

// When the exception is thrown above, the value of `x` is reverted to the
// value before the `try` block. Value semantics is very important here. Without
// it, you'd need to do something much more heavy like snapshot the entire
// virtual machine. In ValueScript, we just take a snapshot of each variable
// that is mutated inside the `try` block and restore it on `catch`. You can see
// this in the assembly as `%snap_x`.
//
// Note: Method calls don't yet generate snapshots. This should be fixed soon.
// In the meantime you can workaround it by adding `myClass = myClass;`.