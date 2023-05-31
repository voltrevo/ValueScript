//! test_output([0,1])

export default function () {
  return [
    test(true),
    test(false),
  ];
}

function test(shouldThrow: boolean) {
  let x = 0;

  try {
    x++;

    if (shouldThrow) {
      throw new Error("boom");
    }
  } catch {}

  return x;
}
