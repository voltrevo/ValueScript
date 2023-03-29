export default function assert(
  value: boolean,
  msg = 'value was not true',
): asserts value {
  if (value !== true) {
    throw new Error(`Assertion failed: ${msg}`);
  }
}
