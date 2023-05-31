//! test_output(E: ["rethrow",Error{"message":"Something went wrong"}])

export default function () {
  try {
    throw new Error("Something went wrong");
  } catch (error) {
    throw ["rethrow", error];
  }
}
