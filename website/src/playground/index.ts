import monacoPromise from "./monacoPromise.ts";

import files from "./files.ts";
import assert from "./helpers/assert.ts";
import nil from "./helpers/nil.ts";
import notNil from "./helpers/notNil.ts";
import VslibPool, { Job } from "./vslib/VslibPool.ts";

function domQuery<T = HTMLElement>(query: string): T {
  return <T> <unknown> notNil(document.querySelector(query) ?? nil);
}

const editorEl = domQuery("#editor");

const selectEl = domQuery<HTMLSelectElement>("#file-location select");
const filePreviousEl = domQuery("#file-previous");
const fileNextEl = domQuery("#file-next");
const outcomeEl = domQuery("#outcome");
const vsmEl = domQuery("#vsm");

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

  let compileJob: Job<string> | nil = nil;
  let runJob: Job<string> | nil = nil;
  let updateId = 0;

  function handleUpdate() {
    updateId++;
    const currentUpdateId = updateId;
    compileJob?.cancel();
    runJob?.cancel();

    const source = model.getValue();

    compileJob = vslibPool.compile(source);
    runJob = vslibPool.run(source);

    renderJob(compileJob, vsmEl);
    renderJob(runJob, outcomeEl);

    function renderJob(job: Job<string>, el: HTMLElement) {
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
          el.textContent = await job.wait();
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
})();
