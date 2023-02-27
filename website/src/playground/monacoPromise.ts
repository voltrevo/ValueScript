/// <reference types="https://esm.sh/v99/monaco-editor@0.34.1/esm/vs/editor/editor.api.d.ts" />

import * as MonacoImport from "https://esm.sh/v99/monaco-editor@0.34.1/esm/vs/editor/editor.api.d.ts";
// export * from "https://esm.sh/v99/monaco-editor@0.34.1/esm/vs/editor/editor.api.d.ts";

const script = document.createElement("script");
script.src = "/monaco/monaco.bundle.js";
document.head.append(script);

const monacoPromise = new Promise<typeof MonacoImport>((resolve, reject) => {
  script.onload = () => {
    // deno-lint-ignore no-explicit-any
    const monaco = (globalThis as any).monaco;

    if (monaco === undefined) {
      throw new Error("Missing monaco definition");
    }

    resolve(monaco);
  };

  script.onerror = (evt) => {
    reject(new Error(evt.toString()));
  };
});

export default monacoPromise;
