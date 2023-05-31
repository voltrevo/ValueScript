//! test_output("that thing")

export default function () {
  while (true) {
    try {
      return "this thing";
    } finally {
      break;
    }
  }

  try {
  } finally {
  }

  return "that thing";
}
