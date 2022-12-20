import { Semaphore } from 'wait-your-turn';

import WorkerSlot from './WorkerSlot';

export default class WorkerPool {
  #semaphore: Semaphore;
  #slots: WorkerSlot[];

  constructor(
    public scriptUrl: string | URL,
    public size = globalThis.navigator?.hardwareConcurrency ?? 4,
  ) {
    this.#semaphore = new Semaphore(this.size);
    this.#slots = [];

    for (let i = 0; i < this.size; i++) {
      this.#slots.push(new WorkerSlot(scriptUrl));
    }

    this.#slots[0].start();
  }

  async use<T>(fn: (worker: Worker, terminate: () => void) => T): Promise<T> {
    return await this.#semaphore.use(async () => {
      let bestSlot = this.#slots[0];

      for (let i = 1; i < this.size; i++) {
        const slot = this.#slots[i];
  
        if (this.#slots[i].useCount < this.#slots[i].useCount) {
          bestSlot = slot;
        } else if (this.#slots[i].useCount === this.#slots[i].useCount) {
          const stateScoreMap = {
            empty: 0,
            starting: 1,
            started: 2,
          };
  
          if (stateScoreMap[slot.state] > stateScoreMap[bestSlot.state]) {
            bestSlot = slot;
          }
        }
      }

      if (bestSlot.useCount > 0) {
        console.error(
          'Best slot is already in use (should be prevented via semaphore)',
        );
      }

      return await bestSlot.use(fn);
    });
  }
}
