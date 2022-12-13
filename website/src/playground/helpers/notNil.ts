import nil from "./nil.ts";

export default function notNil<T>(value: T | nil): T {
  if (value === nil) {
    throw new Error();
  }

  return value;
}
