import { Mutex } from "wait-your-turn";

import nil from "./helpers/nil";

export default class WorkerSlot {
  worker?: Worker;
  mutex = new Mutex();
  state: 'empty' | 'starting' | 'started' = 'empty';
  useCount = 0;

  constructor(public scriptUrl: string | URL) {}

  async start() {
    await this.mutex.use(() => this.#Worker());
  }

  async use<T>(fn: (worker: Worker, terminate: () => void) => T): Promise<T> {
    this.useCount++;
    const release = await this.mutex.acquire();

    let finish = () => {
      finish = () => {};
      release();
      this.useCount--;
    }

    try {
      const worker = await this.#Worker();

      return await fn(worker, () => {
        worker.terminate();
        this.worker = nil;
        this.state = 'empty';
        finish();
      });
    } finally {
      finish();
    }
  }

  async #Worker() {
    if (this.worker) {
      return this.worker;
    }

    this.state = 'starting';

    const worker = new Worker(this.scriptUrl);

    await new Promise<void>((resolve, reject) => {
      worker.onmessage = evt => {
        if (evt.data === "ready") {
          resolve();
        } else {
          this.state = 'empty';
          worker.terminate();

          reject(new Error(
            `Unexpected initial message from worker: ${evt.data}`,
          ));
        }
      };
    });

    worker.onmessage = null;
    this.worker = worker;

    this.state = 'started';

    return worker;
  }
}
