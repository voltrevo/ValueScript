export default function () {
  let x = 0;

  try {
    x++;
    throw new Error("boom");
  } catch {}

  return x;
  // TypeScript:  1
  // ValueScript: 0
}

// In ValueScript, a try block is a transaction - it either runs to
// completion, or it is reverted. This is impractical in TypeScript,
// but in ValueScript all we have to do is snapshot the variables and
// restore from them on catch. This works because all mutation is
// local - nothing else can be affected.