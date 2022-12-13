import { initVslib } from "./vslib/index.ts";

initVslib().then((vslib) => {
  (globalThis as any).vslib = vslib;
});
