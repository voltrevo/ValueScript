//! test_error(["outer finally",Error{"message":"inner finally"}])

export default function () {
  let logs: unknown[] = [];

  try {
    try {
      try {
        throw new Error("nested error");
      } finally {
        throw new Error("inner finally");
      }
    } finally {
      logs.push("outer finally");
    }
  } catch (error) {
    logs.push(error);
  }

  return logs;
}
