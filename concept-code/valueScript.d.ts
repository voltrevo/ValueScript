declare module 'value-script' {
  export function staticAssert(value: boolean): asserts value;
  export function lessThan(left: unknown, right: unknown): boolean;
  export function inc(this: number): void;
}
