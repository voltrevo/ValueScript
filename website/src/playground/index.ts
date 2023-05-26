import assert from "./helpers/assert.ts";
import nil from "./helpers/nil.ts";
import notNil from "./helpers/notNil.ts";
import VslibPool, {
  CompilerOutput,
  Diagnostic,
  Job,
  RunResult,
} from "./vslib/VslibPool.ts";
import FileSystem from "./FileSystem.ts";
import monaco from "./monaco.ts";
import Swal from "./Swal.ts";
import { defaultFiles } from "./files/index.ts";
import hasExtension from "./helpers/hasExtension.ts";
import ExplicitAny from "./helpers/ExplicitAny.ts";

function domQuery<T = HTMLElement>(query: string): T {
  return <T>(<unknown>notNil(document.querySelector(query) ?? nil));
}

const editorLoadingEl = domQuery("#editor-loading");
const monacoEditorEl = domQuery("#monaco-editor");
const fileListEl = domQuery("#file-list");

const fileLocationText = domQuery("#file-location-text");

const listBtn = domQuery("#list-btn");
const renameBtn = domQuery("#rename-btn");
const addBtn = domQuery("#add-btn");
const restoreBtn = domQuery("#restore-btn");
const deleteBtn = domQuery("#delete-btn");

const filePreviousEl = domQuery("#file-previous");
const fileNextEl = domQuery("#file-next");
const outcomeEl = domQuery("#outcome");
const vsmEl = domQuery("#vsm");
const diagnosticsEl = domQuery("#diagnostics");

let currentFile = "";

(async () => {
  const vslibPool = new VslibPool();
  const fs = new FileSystem();

  const fileModels = Object.fromEntries(
    fs.list.map((filename) => {
      const content = fs.read(filename);
      assert(content !== nil);

      const model = monaco.editor.createModel(
        content,
        "typescript",
        monaco.Uri.parse(filename),
      );

      model.updateOptions({ tabSize: 2, insertSpaces: true });

      return [filename, model];
    }),
  );

  editorLoadingEl.remove();

  const editor = monaco.editor.create(monacoEditorEl, {
    theme: "vs-dark",
    language: "typescript",
  });

  {
    const editorService = (editor as ExplicitAny)._codeEditorService;
    const openEditorBase = editorService.openCodeEditor.bind(editorService);
    editorService.openCodeEditor = async (
      input: ExplicitAny,
      source: ExplicitAny,
    ) => {
      const result = await openEditorBase(input, source);

      if (result === null) {
        changeFile(input.resource.path);
        editor.setSelection(input.options.selection);
      }

      return result; // always return the base result
    };
  }

  setTimeout(() => changeFile(location.hash.slice(1)));

  globalThis.addEventListener("hashchange", () => {
    changeFile(location.hash.slice(1));
  });

  globalThis.addEventListener("resize", () => editor.layout());

  function changeFile(newFile: string) {
    if (currentFile === "") {
      currentFile = fs.list[0];
    } else if (newFile === currentFile) {
      return;
    }

    if (newFile === "") {
      newFile = fs.list[0];
    }

    const fileIdx = fs.list.indexOf(newFile);

    if (fileIdx !== -1) {
      currentFile = newFile;
    }

    history.replaceState(null, "", `#${currentFile}`);
    fileLocationText.textContent = currentFile;

    const model = fileModels[currentFile];

    editor.setModel(model);
    handleUpdate();

    if (Object.keys(defaultFiles).includes(currentFile)) {
      renameBtn.classList.add("disabled");
      restoreBtn.classList.remove("disabled");
      deleteBtn.classList.add("disabled");
    } else {
      renameBtn.classList.remove("disabled");
      restoreBtn.classList.add("disabled");
      deleteBtn.classList.remove("disabled");
    }
  }

  const moveFileIndex = (change: number) => () => {
    let idx = fs.list.indexOf(currentFile);

    if (idx === -1) {
      throw new Error("This should not happen");
    }

    idx += change;
    idx = Math.max(idx, 0);
    idx = Math.min(idx, fs.list.length - 1);

    changeFile(fs.list[idx]);
  };

  filePreviousEl.addEventListener("click", moveFileIndex(-1));
  fileNextEl.addEventListener("click", moveFileIndex(1));

  let timerId: nil | number = nil;

  editor.onDidChangeModelContent(() => {
    clearTimeout(timerId);
    timerId = setTimeout(handleUpdate, 100) as unknown as number;
  });

  let compileJob: Job<CompilerOutput> | nil = nil;
  let runJob: Job<RunResult> | nil = nil;
  let updateId = 0;

  function handleUpdate() {
    updateId++;
    const currentUpdateId = updateId;
    compileJob?.cancel();
    runJob?.cancel();

    const source = editor.getValue();
    fs.write(currentFile, source);

    compileJob = vslibPool.compile(currentFile, fs.files);
    runJob = vslibPool.run(currentFile, fs.files, []);

    renderJob(compileJob, vsmEl, (el, compilerOutput) => {
      el.textContent = compilerOutput.assembly.join("\n");
    });

    renderJob(runJob, outcomeEl, (el, runResult) => {
      if ("Ok" in runResult.output) {
        el.textContent = runResult.output.Ok;
      } else if ("Err" in runResult.output) {
        el.textContent = `Uncaught exception: ${runResult.output.Err}`;
      } else {
        never(runResult.output);
      }

      diagnosticsEl.innerHTML = "";

      for (const [file, diagnostics] of Object.entries(runResult.diagnostics)) {
        for (const diagnostic of diagnostics) {
          const diagnosticEl = document.createElement("div");

          diagnosticEl.classList.add(
            "diagnostic",
            toKebabCase(diagnostic.level),
          );

          const { line, col } = toLineCol(source, diagnostic.span.start);
          diagnosticEl.textContent = `${file}:${line}:${col}: ${diagnostic.message}`;

          diagnosticsEl.appendChild(diagnosticEl);
        }
      }

      const model = editor.getModel();
      assert(model !== null);

      monaco.editor.setModelMarkers(
        model,
        "valuescript",
        Object.entries(runResult.diagnostics)
          .map(([file, diagnostics]) => {
            if (file !== currentFile) {
              return []; // TODO
            }

            return diagnostics.map((diagnostic) => {
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
            });
          })
          .flat(),
      );
    });

    function renderJob<T>(
      job: Job<T>,
      el: HTMLElement,
      apply: (el: HTMLElement, jobResult: T) => void,
    ) {
      const startTime = Date.now();

      const loadingInterval = setInterval(() => {
        if (currentUpdateId === updateId) {
          el.textContent = `Loading... ${(
            (Date.now() - startTime) /
            1000
          ).toFixed(1)}s`;
        }
      }, 100);

      (async () => {
        try {
          apply(el, await job.wait());
          el.classList.remove("error");
        } catch (err: ExplicitAny) {
          if (!(err instanceof Error)) {
            // eslint-disable-next-line no-ex-assign
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

  function toMonacoSeverity(level: Diagnostic["level"]) {
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

  addBtn.onclick = async () => {
    const popup = await Swal.fire({
      title: "New File",
      input: "text",
      inputPlaceholder: "Enter name",
    });

    if (typeof popup.value === "string" && popup.value !== "") {
      const newFile = sanitizeNewPath(popup.value);

      if (fs.list.includes(newFile)) {
        changeFile(newFile);
      } else {
        fs.write(newFile, "", currentFile);

        fileModels[newFile] = monaco.editor.createModel(
          "",
          "typescript",
          monaco.Uri.parse(newFile),
        );

        changeFile(newFile);
      }
    }
  };

  restoreBtn.onclick = async () => {
    const defaultContent = defaultFiles[currentFile];

    if (defaultContent !== undefined) {
      editor.setValue(defaultContent);
    }
  };

  renameBtn.onclick = async () => {
    if (renameBtn.classList.contains("disabled")) {
      return;
    }

    const currentParts = currentFile.split("/");

    const prefill =
      currentParts.length === 1
        ? ""
        : `${currentParts.slice(0, -1).join("/")}/`;

    const popup = await Swal.fire({
      title: "Rename File",
      input: "text",
      inputValue: prefill,
      inputPlaceholder: currentFile,
    });

    if (typeof popup.value === "string" && popup.value !== "") {
      const newFile = sanitizeNewPath(popup.value);
      fs.rename(currentFile, newFile);
      const model = fileModels[currentFile];
      delete fileModels[currentFile];
      fileModels[newFile] = model;
      changeFile(newFile);
    }
  };

  deleteBtn.onclick = async () => {
    if (deleteBtn.classList.contains("disabled")) {
      return;
    }

    const popup = await Swal.fire({
      title: "Delete File",
      text: `Are you sure you want to delete ${currentFile}?`,
      icon: "warning",
      showCancelButton: true,
      confirmButtonText: "Delete",
      cancelButtonText: "Cancel",
    });

    if (popup.isConfirmed) {
      const idx = Math.max(0, fs.list.indexOf(currentFile) - 1);
      fs.write(currentFile, nil);
      fileModels[currentFile].dispose();
      delete fileModels[currentFile];
      changeFile(fs.list[idx]);
    }
  };

  listBtn.onclick = async () => {
    monacoEditorEl.style.display = "none";

    fileListEl.textContent = "";

    fileListEl.appendChild(makeFileSpacer("0.5em"));
    let currentEl: HTMLElement | undefined;

    for (const file of fs.list) {
      const fileEl = document.createElement("div");
      fileEl.classList.add("file");
      fileEl.textContent = file;

      if (file === currentFile) {
        fileEl.classList.add("current");
        currentEl = fileEl;
      }

      fileEl.onclick = () => {
        changeFile(file);
        monacoEditorEl.style.display = "";
        fileListEl.style.display = "";
      };

      fileListEl.appendChild(fileEl);
    }

    fileListEl.appendChild(makeFileSpacer("1.5em"));

    fileListEl.style.display = "flex";

    if (currentEl) {
      currentEl.scrollIntoView();
    }
  };
})();

function makeFileSpacer(minHeight: string) {
  const spacer = document.createElement("div");
  spacer.classList.add("file-spacer");
  spacer.style.minHeight = minHeight;
  return spacer;
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
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

function sanitizeNewPath(path: string) {
  if (!hasExtension(path)) {
    path = `${path}.ts`;
  }

  if (!path.startsWith("/")) {
    path = `/${path}`;
  }

  return path;
}
