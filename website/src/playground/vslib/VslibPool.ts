import * as valuescript from 'valuescript';
import nil from '../helpers/nil';
import { initVslib } from './index';

async function main() {
  const vslib = await initVslib();
  const nil = undefined;

  self.postMessage('ready');

  self.onmessage = (evt) => {
    const { method, args } = evt.data;

    if (method === 'compile') {
      try {
        self.postMessage({ ok: vslib.compile(args[0]) });
      } catch (err) {
        self.postMessage({ err });
      }
    }

    if (method === 'run') {
      try {
        self.postMessage({ ok: vslib.run(args[0]) });
      } catch (err) {
        self.postMessage({ err });
      }
    }

    if (method === 'runLinked') {
      const [entryPoint, files] = args;

      try {
        self.postMessage({
          ok: vslib.runLinked(entryPoint, filePath => {
            let content = files[filePath];

            if (content === nil && !filePath.endsWith('.ts')) {
              content = files[`${filePath}.ts`];
            }

            if (content === nil) {
              throw new Error('Not found');
            }

            return content;
          }),
        });
      } catch (err) {
        self.postMessage({ err });
      }
    }
  };
}

const workerScript = [
  initVslib.toString(),
  `(${main.toString()})()`,
].join('\n\n');

const workerUrl = URL.createObjectURL(
  new Blob([workerScript], { type: 'application/javascript' }),
);

export type Diagnostic = {
  level: 'Lint' | 'Error' | 'InternalError' | 'CompilerDebug';
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
  output:
    | { Ok: string }
    | { Err: string };
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

  run(source: string) {
    return this.#Job('run', [source]) as Job<RunResult>;
  }

  runLinked(entryPoint: string, files: Record<string, string | nil>) {
    return this.#Job('runLinked', [entryPoint, files]) as Job<RunResult>;
  }

  compile(source: string) {
    return this.#Job('compile', [source]) as Job<CompilerOutput>;
  }

  #Job(method: string, args: unknown[]) {
    let canceled = false;
    let finished = false;
    let outerTerminate: (() => void) | nil = nil;

    const resultPromise = this.#pool.use((worker, terminate) => {
      if (canceled) {
        finished = true;
        return Promise.reject(new Error('canceled'));
      }

      outerTerminate = terminate;

      return new Promise((resolve, reject) => {
        worker.postMessage({ method, args });

        worker.onmessage = (evt) => {
          if ('ok' in evt.data) {
            resolve(JSON.parse(evt.data.ok));
          } else if ('err' in evt.data) {
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
