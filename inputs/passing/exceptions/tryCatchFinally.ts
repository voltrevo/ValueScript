//! test_output(E: Error{"message":"teraboom"})

export default function () {
  try {
    throw new Error("boom");
  } catch {
    throw new Error("megaboom");
  } finally {
    throw new Error("teraboom");
  }
}
