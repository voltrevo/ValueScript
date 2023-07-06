//! test_output(E: {"code":400,"message":"Bad request"})

export default function () {
  throw new CustomError(400, "Bad request");
}

class CustomError /* extends Error */ {
  constructor(
    public code: number,
    public message: string,
  ) {}
}
