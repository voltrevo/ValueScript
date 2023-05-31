//! test_output("Caught: boom")

export default function () {
  try {
    callBoom();
  } catch (error) {
    return `Caught: ${error.message}`;
  }
}

function callBoom() {
  boom();
}

function boom() {
  throw new Error("boom");
}
