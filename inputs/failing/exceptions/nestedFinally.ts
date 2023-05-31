//! test_output([Error{"message":"inner finally"}])
//
// This is wrong. It should be:
// //(!) test_output(["outer finally",Error{"message":"inner finally"}])

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
