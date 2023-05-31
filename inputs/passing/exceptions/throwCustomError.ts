//! test_error(E: {"code":"400","message":"Bad request"})

export default function () {
  throw new CustomError(400, "Bad request");
}

class CustomError /* extends Error */ {
  code: number;
  message: string;

  constructor(code: number, message: string) {
    this.code = code;
    this.message = message;
  }
}
