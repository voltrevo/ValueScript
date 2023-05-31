//! test_output("🤷‍♂️")

export default function () {
  while (true) {
    try {
      throw new Error("boom?");
    } finally {
      break;
    }
  }

  return "🤷‍♂️";
}
