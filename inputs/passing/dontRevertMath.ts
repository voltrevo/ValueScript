//! test_output(undefined)

export default function () {
  try {
    Math.sin(1);
    throw new Error("boom");
  } catch {}
}
