//! test_output(["matched 43","matched 42"])

export default function () {
  let logs: string[] = [];

  switch (21 + 22) {
    case 1:
      logs.push("matched 1");
      break;

    case 43:
      logs.push("matched 43");
      // falls through
    case 42:
      logs.push("matched 42");
      break;

    default:
      logs.push("default");
  }

  return logs;
}
