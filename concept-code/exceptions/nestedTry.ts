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
