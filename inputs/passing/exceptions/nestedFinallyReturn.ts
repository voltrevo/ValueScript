// test_output! "this thing"

export default function () {
  while (true) {
    try {
      try {
        return "this thing";
      } finally {
        1 + 1;
      }
    } finally {
      2 + 2;
    }

    break;
  }

  try {
  } finally {
  }

  return "that thing";
}
