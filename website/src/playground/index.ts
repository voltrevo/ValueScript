import monacoPromise from "./monacoPromise.ts";

import files from "./files.ts";
import assert from "./helpers/assert.ts";
import nil from "./helpers/nil.ts";
import notNil from "./helpers/notNil.ts";
import { initVslib } from "./vslib/index.ts";

function domQuery<T = HTMLElement>(query: string): T {
  return <T> <unknown> notNil(document.querySelector(query) ?? nil);
}

const editorEl = domQuery("#editor");

const selectEl = domQuery<HTMLSelectElement>("#file-location select");
const filePreviousEl = domQuery("#file-previous");
const fileNextEl = domQuery("#file-next");
const vsmEl = domQuery("#vsm");

for (const filename of Object.keys(files)) {
  const option = document.createElement("option");
  option.textContent = filename;
  selectEl.appendChild(option);
}

let currentFile = "";

editorEl.innerHTML = "";

(async () => {
  const [vslib, monaco] = await Promise.all([
    initVslib(),
    monacoPromise,
  ]);

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

    timerId = setTimeout(handleUpdate, 200);
  });

  function handleUpdate() {
    vsmEl.textContent = vslib.compile(model.getValue());
  }
})();
