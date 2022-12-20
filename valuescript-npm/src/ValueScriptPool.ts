import WorkerPool from "./WorkerPool";

type Job<Result> = {
  wait: () => Promise<Result>,
  cancel: () => void,
};

type JobMethod<Params extends unknown[], Result> = (
  ...params: Params
) => Job<Result>;

type Api = {
  compile: JobMethod<[string], string>,
  run: JobMethod<[string], unknown>,
};

export default class ValueScriptPool implements Api {
  #workerPool: WorkerPool;

  constructor(
    wasmUrl = 'todo',
  ) {
    throw new Error('Not implemented');
  }

  compile: JobMethod<[string], string>;
  run: JobMethod<[string], unknown>;
}
