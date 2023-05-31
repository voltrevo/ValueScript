export default function () {
  try {
    throw new Error("Something went wrong");
  } finally {
    1 + 1;
  }
}
