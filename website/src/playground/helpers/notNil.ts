import nil from "./nil";

export default function notNil<T>(value: T | nil): T {
  if (value === nil) {
    throw new Error();
  }

  return value;
}
