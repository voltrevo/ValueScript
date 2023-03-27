import files from './files';
import assert from './helpers/assert';
import nil from './helpers/nil';
import notNil from './helpers/notNil';
import VslibPool, {
  CompilerOutput,
  Diagnostic,
  Job,
  RunResult,
} from './vslib/VslibPool';
import FileSystem from './FileSystem';
import monaco from './monaco';

function domQuery<T = HTMLElement>(query: string): T {
  return <T> <unknown> notNil(document.querySelector(query) ?? nil);
}

const editorEl = domQuery('#editor');

const fileLocation = domQuery<HTMLSelectElement>('#file-location');
const filePreviousEl = domQuery('#file-previous');
const fileNextEl = domQuery('#file-next');
const outcomeEl = domQuery('#outcome');
const vsmEl = domQuery('#vsm');
const diagnosticsEl = domQuery('#diagnostics');

let currentFile = '';

editorEl.innerHTML = '';

(async () => {
  const vslibPool = new VslibPool();

  (window as any).vslibPool = vslibPool;

  const fs = new FileSystem(files);

  const fileModels = Object.fromEntries(fs.list.map(
    (filename) => {
      const content = fs.read(filename);
      assert(content !== nil);

      const model = monaco.editor.createModel(
        content,
        'typescript',
        monaco.Uri.parse(filename),
      );

      model.updateOptions({ tabSize: 2, insertSpaces: true });

      return [filename, model];
    },
  ));

  const editor = monaco.editor.create(editorEl, {
    theme: 'vs-dark',
    language: 'typescript',
  });

  {
    const editorService = (editor as any)._codeEditorService;
    const openEditorBase = editorService.openCodeEditor.bind(editorService);
    editorService.openCodeEditor = async (input: any, source: any) => {
      const result = await openEditorBase(input, source);

      if (result === null) {
        changeFile(input.resource.path.slice(1));
        editor.setSelection(input.options.selection);
      }

      return result; // always return the base result
    };
  }

  setTimeout(() => changeFile(location.hash.slice(1)));

  globalThis.addEventListener('hashchange', () => {
    changeFile(location.hash.slice(1));
  });

  globalThis.addEventListener('resize', () => editor.layout());

  function changeFile(newFile: string) {
    if (currentFile === '') {
      currentFile = Object.keys(files)[0];
    } else if (newFile === currentFile) {
      return;
    }

    if (newFile === '') {
      newFile = Object.keys(files)[0];
    }

    const fileIdx = Object.keys(files).indexOf(newFile);

    if (fileIdx !== -1) {
      currentFile = newFile;
    }

    location.hash = currentFile;
    fileLocation.textContent = currentFile;

    const model = fileModels[currentFile];

    editor.setModel(model);
    handleUpdate();
  }

  const moveFileIndex = (change: number) => () => {
    let idx = fs.list.indexOf(currentFile);

    if (idx === -1) {
      throw new Error('This should not happen');
    }

    idx += change;
    idx = Math.max(idx, 0);
    idx = Math.min(idx, fs.list.length - 1);

    changeFile(fs.list[idx]);
  };

  filePreviousEl.addEventListener('click', moveFileIndex(-1));
  fileNextEl.addEventListener('click', moveFileIndex(1));

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

    compileJob = vslibPool.compile(source);
    runJob = vslibPool.run(source);

    renderJob(
      compileJob,
      vsmEl,
      (el, compilerOutput) => {
        el.textContent = compilerOutput.assembly.join('\n');
      },
    );

    renderJob(
      runJob,
      outcomeEl,
      (el, runResult) => {
        if ('Ok' in runResult.output) {
          el.textContent = runResult.output.Ok;
        } else if ('Err' in runResult.output) {
          el.textContent = `Uncaught exception: ${runResult.output.Err}`;
        } else {
          never(runResult.output);
        }

        diagnosticsEl.innerHTML = '';

        for (const diagnostic of runResult.diagnostics) {
          const diagnosticEl = document.createElement('div');

          diagnosticEl.classList.add(
            'diagnostic',
            toKebabCase(diagnostic.level),
          );

          const { line, col } = toLineCol(source, diagnostic.span.start);
          diagnosticEl.textContent = `${line}:${col}: ${diagnostic.message}`;

          diagnosticsEl.appendChild(diagnosticEl);
        }

        const model = editor.getModel();
        assert(model !== null);

        monaco.editor.setModelMarkers(
          model,
          'valuescript',
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
          el.classList.remove('error');
        } catch (err: any) {
          if (!(err instanceof Error)) {
            // deno-lint-ignore no-ex-assign
            err = new Error(`Non-error exception ${err}`);
          }

          if (err.message !== 'Canceled') {
            el.textContent = err.message;
            el.classList.add('error');
          }
        } finally {
          clearInterval(loadingInterval);
        }
      })();
    }
  }

  function toMonacoSeverity(level: Diagnostic['level']) {
    switch (level) {
    case 'Error':
      return monaco.MarkerSeverity.Error;
    case 'InternalError':
      return monaco.MarkerSeverity.Error;
    case 'Lint':
      return monaco.MarkerSeverity.Warning;
    case 'CompilerDebug':
      return monaco.MarkerSeverity.Info;
    }
  }
})();

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function never(_: never): never {
  throw new Error('This should not happen');
}

function toKebabCase(str: string): string {
  // account for leading capital letters
  str = str.replace(/^[A-Z]/, (match) => match.toLowerCase());

  return str.replace(/[A-Z]/g, (match) => `-${match.toLowerCase()}`);
}

function toLineCol(str: string, index: number): { line: number; col: number } {
  const lines = str.slice(0, index).split('\n');

  return { line: lines.length, col: lines[lines.length - 1].length + 1 };
}
