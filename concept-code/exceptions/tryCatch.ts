export default function () {
  try {
    throw new Error("Something went wrong");
  } catch (error) {
    return error;
  }
}
