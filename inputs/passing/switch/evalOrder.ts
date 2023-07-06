//! test_output(["c1","c2","default"])

export default function foo() {
  let logs = [];

  switch (0) {
    case logs.push("c1"):
      break;
    default:
      logs.push("default");
      break;
    case logs.push("c2"):
      break;
  }

  return logs;
}
