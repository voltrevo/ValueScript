/// <reference path="../../vs.d.ts" />

import f1 from "./f1.ts";
import { f2 } from "./f2.ts";
import * as util from "./util.ts";

export default function main() {
  let x = f1(1, 2, 3);
  vs.apply(f2, x, []);

  return [x, util.dist(3, 4)];
}
