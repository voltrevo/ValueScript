//! test_output(["matched 42"])

export default function foo() {
  let logs: string[] = [];

  switch (21 + 21) {
    case 1:
      logs.push("matched 1");
      break;

    default:
      logs.push("default");
      // falls through
    case 42:
      logs.push("matched 42");
  }

  return logs;
}
