//! test_output(E: Error{"message":"Something went wrong"})

export default function () {
  try {
    throw new Error("Something went wrong");
  } finally {
    1 + 1;
  }
}
