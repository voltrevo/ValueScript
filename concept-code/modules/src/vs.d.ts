declare namespace vs {
  /** compiles to `apply %fn %this_ %args %output` */
  export function apply(fn: unknown, this_: unknown, args: unknown[]): unknown;

  /** compiles to `op++ %x` */
  export function inc(x: number): void;
}

export default vs;
