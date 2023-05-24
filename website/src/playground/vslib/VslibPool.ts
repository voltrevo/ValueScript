import * as valuescript from "valuescript";
import nil from "../helpers/nil";
import { initVslib } from "./index";

async function main() {
  const vslib = await initVslib();

  self.postMessage("ready");

  self.onmessage = (evt) => {
    const { method, args } = evt.data;

    if (method === "compile") {
      const [entryPoint, files] = args;

      try {
        self.postMessage({
          ok: vslib.compile(entryPoint, makeLookupFile(files)),
        });
      } catch (err) {
        self.postMessage({ err });
      }
    }

    if (method === "run") {
      const [entryPoint, files, argsStr] = args;

      try {
        self.postMessage({
          ok: vslib.run(entryPoint, makeLookupFile(files), argsStr),
        });
      } catch (err) {
        self.postMessage({ err });
      }
    }
  };

  function makeLookupFile(files: Record<string, string | nil>) {
    return (filePath: string) => {
      let content = files[filePath];

      if (content === undefined && !hasExtension(filePath)) {
        content = files[`${filePath}.ts`] ?? files[`${filePath}.js`];
      }

      if (content === undefined) {
        throw new Error("Not found");
      }

      return content;
    };
  }

  function hasExtension(path: string) {
    return path.split("/").at(-1)?.includes(".") ?? false;
  }
}

const workerScript = [initVslib.toString(), `(${main.toString()})()`].join(
  "\n\n",
);

const workerUrl = URL.createObjectURL(
  new Blob([workerScript], { type: "application/javascript" }),
);

export type Diagnostic = {
  level: "Lint" | "Error" | "InternalError" | "CompilerDebug";
  message: string;
  span: {
    start: number;
    end: number;
    ctxt: number;
  };
};

export type CompilerOutput = {
  diagnostics: Record<string, Diagnostic[]>;
  assembly: string[];
};

export type RunResult = {
  diagnostics: Record<string, Diagnostic[]>;
  output: { Ok: string } | { Err: string };
};

export type Job<T> = {
  wait: () => Promise<T>;
  cancel: () => void;
};

export function mapJob<U, V>(job: Job<U>, f: (x: U) => V): Job<V> {
  return {
    wait: () => job.wait().then(f),
    cancel: job.cancel,
  };
}

export default class VslibPool {
  #pool = new valuescript.WorkerPool(workerUrl);

  run(
    entryPoint: string,
    files: Record<string, string | nil>,
    args: unknown[],
  ) {
    return this.#Job("run", [
      entryPoint,
      files,
      JSON.stringify(args),
    ]) as Job<RunResult>;
  }

  compile(entryPoint: string, files: Record<string, string | nil>) {
    return this.#Job("compile", [entryPoint, files]) as Job<CompilerOutput>;
  }

  #Job(method: string, args: unknown[]) {
    let canceled = false;
    let finished = false;
    let outerTerminate: (() => void) | nil = nil;

    const resultPromise = this.#pool.use((worker, terminate) => {
      if (canceled) {
        finished = true;
        return Promise.reject(new Error("canceled"));
      }

      outerTerminate = terminate;

      return new Promise((resolve, reject) => {
        worker.postMessage({ method, args });

        worker.onmessage = (evt) => {
          if ("ok" in evt.data) {
            resolve(JSON.parse(evt.data.ok));
          } else if ("err" in evt.data) {
            if (evt.data.err instanceof Error) {
              reject(evt.data.err);
            } else {
              reject(new Error(`${evt.data.err}`));
            }
          } else {
            reject(new Error(`Unexpected message: ${evt.data}`));
          }

          finished = true;
        };
      });
    }) as Promise<unknown>;

    return {
      wait: () => resultPromise,
      cancel: () => {
        canceled = true;

        if (!finished && outerTerminate) {
          outerTerminate();
        }
      },
    };
  }
}
