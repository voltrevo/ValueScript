//! test_output([[],["item"]])

export default function () {
  return [
    test(true),
    test(false),
  ];
}

function test(shouldThrow: boolean) {
  let x = [];

  try {
    x.push("item");

    if (shouldThrow) {
      throw new Error("boom");
    }
  } catch {}

  return x;
}
