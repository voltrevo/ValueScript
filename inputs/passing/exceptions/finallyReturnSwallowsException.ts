// test_output! 42

export default function () {
  try {
    throw new Error("Test error");
  } finally {
    return 42;
  }
}
