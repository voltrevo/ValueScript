//! test_output(0)

export default function main() {
  return measure(true) - measure(false);
}

function measure(swap: boolean) {
  const x = Debug.makeCopyCounter("x");

  let arr = [x, "y", "z"];

  if (swap) {
    [arr[1], arr[2]] = [arr[2], arr[1]];
  }

  return x.count;
}
