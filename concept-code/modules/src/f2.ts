import vs from "./vs.d.ts";

export function f2(this: number) {
  vs.inc(this);
}
