// test_error! Error{"message":"nested error"}

export default function () {
  try {
    try {
      throw new Error("nested error");
    } catch (error) {
      throw error;
    }
  } catch (error) {
    throw error;
  }
}
