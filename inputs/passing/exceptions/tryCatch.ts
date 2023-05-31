//! test_output(Error{"message":"Something went wrong"})

export default function () {
  try {
    throw new Error("Something went wrong");
  } catch (error) {
    return error;
  }
}
