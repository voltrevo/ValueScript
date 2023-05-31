//! test_output(E: [Error{"message":"nested error"}])

export default function main() {
  try {
    try {
      throw new Error("nested error");
    } catch (error) {
      throw error;
    }
  } catch (error) {
    throw [error];
  }
}
