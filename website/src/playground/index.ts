import monacoPromise from "./monacoPromise.ts";

import files from "./files.ts";
import assert from "./helpers/assert.ts";
import nil from "./helpers/nil.ts";
import notNil from "./helpers/notNil.ts";
import VslibPool, {
  CompilerOutput,
  Diagnostic,
  Job,
  RunResult,
} from "./vslib/VslibPool.ts";

function domQuery<T = HTMLElement>(query: string): T {
  return <T> <unknown> notNil(document.querySelector(query) ?? nil);
}

const editorEl = domQuery("#editor");

const selectEl = domQuery<HTMLSelectElement>("#file-location select");
const filePreviousEl = domQuery("#file-previous");
const fileNextEl = domQuery("#file-next");
const outcomeEl = domQuery("#outcome");
const vsmEl = domQuery("#vsm");
const diagnosticsEl = domQuery("#diagnostics");

for (const filename of Object.keys(files)) {
  const option = document.createElement("option");
  option.textContent = filename;
  selectEl.appendChild(option);
}

let currentFile = "";

editorEl.innerHTML = "";

(async () => {
  const vslibPool = new VslibPool();
  const monaco = await monacoPromise;

  (window as any).vslibPool = vslibPool;

  const editor = monaco.editor.create(editorEl, {
    theme: "vs-dark",
    value: "",
    language: "typescript",
  });

  setTimeout(() => changeFile(location.hash.slice(1)));

  globalThis.addEventListener("hashchange", () => {
    changeFile(location.hash.slice(1));
  });

  globalThis.addEventListener("resize", () => editor.layout());

  const model = notNil(editor.getModel() ?? nil);

  model.updateOptions({ tabSize: 2, insertSpaces: true });

  function changeFile(newFile: string) {
    if (currentFile === "") {
      currentFile = Object.keys(files)[0];
    } else if (newFile === currentFile) {
      return;
    }

    if (newFile === "") {
      newFile = Object.keys(files)[0];
    }

    const fileIdx = Object.keys(files).indexOf(newFile);

    if (fileIdx !== -1) {
      currentFile = newFile;
    }

    location.hash = currentFile;
    selectEl.selectedIndex = fileIdx;

    const content = files[currentFile];
    assert(content !== nil);

    model.setValue(content);
  }

  selectEl.addEventListener("change", () => {
    changeFile(selectEl.value);
  });

  const moveFileIndex = (change: number) => () => {
    const filenames = Object.keys(files);
    let idx = filenames.indexOf(currentFile);

    if (idx === -1) {
      throw new Error("This should not happen");
    }

    idx += change;
    idx = Math.max(idx, 0);
    idx = Math.min(idx, filenames.length - 1);

    changeFile(filenames[idx]);
  };

  filePreviousEl.addEventListener("click", moveFileIndex(-1));
  fileNextEl.addEventListener("click", moveFileIndex(1));

  let timerId: undefined | number = undefined;

  model.onDidChangeContent(() => {
    files[currentFile] = model.getValue();
    clearTimeout(timerId);

    timerId = setTimeout(handleUpdate, 100);
  });

  let compileJob: Job<CompilerOutput> | nil = nil;
  let runJob: Job<RunResult> | nil = nil;
  let updateId = 0;

  function handleUpdate() {
    updateId++;
    const currentUpdateId = updateId;
    compileJob?.cancel();
    runJob?.cancel();

    const source = model.getValue();

    compileJob = vslibPool.compile(source);
    runJob = vslibPool.run(source);

    renderJob(
      compileJob,
      vsmEl,
      (el, compilerOutput) => {
        el.textContent = compilerOutput.assembly.join("\n");
      },
    );

    renderJob(
      runJob,
      outcomeEl,
      (el, runResult) => {
        if ("Ok" in runResult.output) {
          el.textContent = runResult.output.Ok;
        } else if ("Err" in runResult.output) {
          el.textContent = runResult.output.Err;
        } else {
          never(runResult.output);
        }

        diagnosticsEl.innerHTML = "";

        for (const diagnostic of runResult.diagnostics) {
          const diagnosticEl = document.createElement("div");

          diagnosticEl.classList.add(
            "diagnostic",
            toKebabCase(diagnostic.level),
          );

          const { line, col } = toLineCol(source, diagnostic.span.start);
          diagnosticEl.textContent = `${line}:${col}: ${diagnostic.message}`;

          diagnosticsEl.appendChild(diagnosticEl);
        }

        monaco.editor.setModelMarkers(
          model,
          "valuescript",
          runResult.diagnostics.map((diagnostic) => {
            const { line, col } = toLineCol(source, diagnostic.span.start);
            const { line: endLine, col: endCol } = toLineCol(
              source,
              diagnostic.span.end,
            );

            return {
              severity: toMonacoSeverity(diagnostic.level),
              startLineNumber: line,
              startColumn: col,
              endLineNumber: endLine,
              endColumn: endCol,
              message: diagnostic.message,
            };
          }),
        );
      },
    );

    function renderJob<T>(
      job: Job<T>,
      el: HTMLElement,
      apply: (el: HTMLElement, jobResult: T) => void,
    ) {
      const startTime = Date.now();

      const loadingInterval = setInterval(() => {
        if (currentUpdateId === updateId) {
          el.textContent = `Loading... ${
            ((Date.now() - startTime) / 1000).toFixed(1)
          }s`;
        }
      }, 100);

      (async () => {
        try {
          apply(el, await job.wait());
          el.classList.remove("error");
        } catch (err) {
          if (!(err instanceof Error)) {
            // deno-lint-ignore no-ex-assign
            err = new Error(`Non-error exception ${err}`);
          }

          if (err.message !== "Canceled") {
            el.textContent = err.message;
            el.classList.add("error");
          }
        } finally {
          clearInterval(loadingInterval);
        }
      })();
    }
  }

  function toMonacoSeverity(level: Diagnostic["level"]): any {
    switch (level) {
      case "Error":
        return monaco.MarkerSeverity.Error;
      case "InternalError":
        return monaco.MarkerSeverity.Error;
      case "Lint":
        return monaco.MarkerSeverity.Warning;
      case "CompilerDebug":
        return monaco.MarkerSeverity.Info;
    }
  }
})();

function never(_: never): never {
  throw new Error("This should not happen");
}

function toKebabCase(str: string): string {
  // account for leading capital letters
  str = str.replace(/^[A-Z]/, (match) => match.toLowerCase());

  return str.replace(/[A-Z]/g, (match) => `-${match.toLowerCase()}`);
}

function toLineCol(str: string, index: number): { line: number; col: number } {
  const lines = str.slice(0, index).split("\n");

  return { line: lines.length, col: lines[lines.length - 1].length + 1 };
}
