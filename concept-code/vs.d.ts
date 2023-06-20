declare namespace vs {
  /** compiles to `apply %fn %this_ %args %output` */
  export function apply(fn: unknown, this_: unknown, args: unknown[]): unknown;

  /** compiles to `op++ %x` */
  export function inc(x: number): void;
}

declare namespace Debug {
  export function log(...args: unknown[]): void;

  export function makeCopyCounter<T = undefined>(
    tag?: T,
  ): { tag: T; count: number };
}

declare module "ffi:console" {
  type Console = {
    log(...args: unknown[]): void;
  };

  const console: Console;

  export default console;
}
