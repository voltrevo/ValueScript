// test_output! ["boom"]
//
// Note: the original intention was for this to output ["here","nested boom","boom"] to check where
// the control flow goes. This worked at the time, but now that snapshotting variables that can be
// mutated via method calls is implemented, the code now correctly reverts those logs and the output
// is just ["boom"].
//
// TODO: Include console.log or Debug.log in tests to enable this checking.

export default function () {
  let logs: unknown[] = [];

  try {
    logs.push("here");

    try {
      throw new Error("nested boom");
    } catch (error) {
      logs.push(error.message);
    }

    throw new Error("boom");
  } catch (error) {
    logs.push(error.message);
  }

  return logs;
}
