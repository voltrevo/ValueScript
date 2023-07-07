//! bench()

// Copyright 2006-2008 the V8 project authors. All rights reserved.
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright
//       notice, this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above
//       copyright notice, this list of conditions and the following
//       disclaimer in the documentation and/or other materials provided
//       with the distribution.
//     * Neither the name of Google Inc. nor the names of its
//       contributors may be used to endorse or promote products derived
//       from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

// This is a JavaScript implementation of the Richards
// benchmark from:
//
//    http://www.cl.cam.ac.uk/~mr10/Bench.html
//
// The benchmark was originally implemented in BCPL by
// Martin Richards.

// var Richards = new BenchmarkSuite("Richards", [35302], [
//   new Benchmark("Richards", true, false, 8200, runRichards),
// ]);

const ID_IDLE = 0;
const ID_WORKER = 1;
const ID_HANDLER_A = 2;
const ID_HANDLER_B = 3;
const ID_DEVICE_A = 4;
const ID_DEVICE_B = 5;
// const NUMBER_OF_IDS = 6;

const KIND_DEVICE = 0;
const KIND_WORK = 1;

/**
 * The Richards benchmark simulates the task dispatcher of an
 * operating system.
 */
export default function runRichards() {
  var scheduler = new Scheduler();
  scheduler.addIdleTask(0, ID_IDLE, null, COUNT);

  var queue = new Packet(null, ID_WORKER, KIND_WORK);
  queue = new Packet(queue, ID_WORKER, KIND_WORK);
  scheduler.addWorkerTask(ID_WORKER, 1000, queue);

  queue = new Packet(null, ID_DEVICE_A, KIND_DEVICE);
  queue = new Packet(queue, ID_DEVICE_A, KIND_DEVICE);
  queue = new Packet(queue, ID_DEVICE_A, KIND_DEVICE);
  scheduler.addHandlerTask(ID_HANDLER_A, 2000, queue);

  queue = new Packet(null, ID_DEVICE_B, KIND_DEVICE);
  queue = new Packet(queue, ID_DEVICE_B, KIND_DEVICE);
  queue = new Packet(queue, ID_DEVICE_B, KIND_DEVICE);
  scheduler.addHandlerTask(ID_HANDLER_B, 3000, queue);

  scheduler.addDeviceTask(ID_DEVICE_A, 4000, null);

  scheduler.addDeviceTask(ID_DEVICE_B, 5000, null);

  scheduler.schedule();

  if (
    scheduler.queueCount != EXPECTED_QUEUE_COUNT ||
    scheduler.holdCount != EXPECTED_HOLD_COUNT
  ) {
    var msg = "Error during execution: queueCount = " + scheduler.queueCount +
      ", holdCount = " + scheduler.holdCount + ".";
    throw new Error(msg);
  }
}

const COUNT = 1000;

/**
 * These two constants specify how many times a packet is queued and
 * how many times a task is put on hold in a correct run of richards.
 * They don't have any meaning a such but are characteristic of a
 * correct run so if the actual queue or hold count is different from
 * the expected there must be a bug in the implementation.
 */
const EXPECTED_QUEUE_COUNT = 2322;
const EXPECTED_HOLD_COUNT = 928;

type SCHEDULER_RELEASE = 0;
type SCHEDULER_HOLD_CURRENT = 1;
type SCHEDULER_SUSPEND_CURRENT = 2;
type SCHEDULER_QUEUE = 3;

type SchedulerAction =
  | [type: SCHEDULER_RELEASE, id: number]
  | [type: SCHEDULER_HOLD_CURRENT]
  | [type: SCHEDULER_SUSPEND_CURRENT]
  | [type: SCHEDULER_QUEUE, packet: Packet];

/**
 * A scheduler can be used to schedule a set of tasks based on their relative
 * priorities.  Scheduling is done by maintaining a list of task control blocks
 * which holds tasks and the data queue they are processing.
 */
class Scheduler {
  queueCount = 0;
  holdCount = 0;

  tcbStore: Record<number, TaskControlBlock> = {};

  // new Array(NUMBER_OF_IDS);
  blocks: (number | null)[] = [null, null, null, null, null, null];

  list: number | null = null;
  currentId: number | null = null;

  /**
   * Add an idle task to this scheduler.
   * @param {int} id the identity of the task
   * @param {int} priority the task's priority
   * @param {Packet} queue the queue of work to be processed by the task
   * @param {int} count the number of times to schedule the task
   */
  addIdleTask(
    id: number,
    priority: number,
    queue: Packet | null,
    count: number,
  ) {
    this.addRunningTask(id, priority, queue, new IdleTask(1, count));
  }

  /**
   * Add a work task to this scheduler.
   * @param {int} id the identity of the task
   * @param {int} priority the task's priority
   * @param {Packet} queue the queue of work to be processed by the task
   */
  addWorkerTask(id: number, priority: number, queue: Packet) {
    this.addTask(
      id,
      priority,
      queue,
      new WorkerTask(ID_HANDLER_A, 0),
    );
  }

  /**
   * Add a handler task to this scheduler.
   * @param {int} id the identity of the task
   * @param {int} priority the task's priority
   * @param {Packet} queue the queue of work to be processed by the task
   */
  addHandlerTask(id: number, priority: number, queue: Packet) {
    this.addTask(id, priority, queue, new HandlerTask());
  }

  /**
   * Add a handler task to this scheduler.
   * @param {int} id the identity of the task
   * @param {int} priority the task's priority
   * @param {Packet} queue the queue of work to be processed by the task
   */
  addDeviceTask(id: number, priority: number, queue: Packet | null) {
    this.addTask(id, priority, queue, new DeviceTask());
  }

  /**
   * Add the specified task and mark it as running.
   * @param {int} id the identity of the task
   * @param {int} priority the task's priority
   * @param {Packet} queue the queue of work to be processed by the task
   * @param {Task} task the task to add
   */
  addRunningTask(
    id: number,
    priority: number,
    queue: Packet | null,
    task: Task,
  ) {
    this.addTask(id, priority, queue, task);
    this.tcbStore[this.currentId!].setRunning();
  }

  /**
   * Add the specified task to this scheduler.
   * @param {int} id the identity of the task
   * @param {int} priority the task's priority
   * @param {Packet} queue the queue of work to be processed by the task
   * @param {Task} task the task to add
   */
  addTask(id: number, priority: number, queue: Packet | null, task: Task) {
    const tcb = new TaskControlBlock(
      this.list,
      id,
      priority,
      queue,
      task,
    );
    this.tcbStore[id] = tcb;
    this.currentId = tcb.id;
    this.list = this.currentId;
    this.blocks[id] = this.currentId;
  }

  /**
   * Execute the tasks managed by this scheduler.
   */
  schedule() {
    this.currentId = this.list;
    while (this.currentId != null) {
      if (this.tcbStore[this.currentId].isHeldOrSuspended()) {
        this.currentId = this.tcbStore[this.currentId].link;
      } else {
        const action = this.tcbStore[this.currentId].run();
        this.currentId = this.processAction(action);
      }
    }
  }

  /**
   * Release a task that is currently blocked and return the next block to run.
   * @param {int} id the id of the task to suspend
   */
  static release(id: number): SchedulerAction {
    return [0, id];
  }

  /**
   * Block the currently executing task and return the next task control block
   * to run.  The blocked task will not be made runnable until it is explicitly
   * released, even if new work is added to it.
   */
  static holdCurrent(): SchedulerAction {
    return [1];
  }

  /**
   * Suspend the currently executing task and return the next task control block
   * to run.  If new work is added to the suspended task it will be made runnable.
   */
  static suspendCurrent(): SchedulerAction {
    return [2];
  }

  /**
   * Add the specified packet to the end of the worklist used by the task
   * associated with the packet and make the task runnable if it is currently
   * suspended.
   * @param {Packet} packet the packet to add
   */
  static queue(packet: Packet): SchedulerAction {
    return [3, packet];
  }

  processAction(action: SchedulerAction): number | null {
    if (action[0] === 0) { // release
      const id = action[1];

      var tcbId = this.blocks[id];
      if (tcbId == null) return tcbId;
      this.tcbStore[tcbId].markAsNotHeld();
      if (
        this.tcbStore[tcbId].priority > this.tcbStore[this.currentId!].priority
      ) {
        return tcbId;
      } else {
        return this.currentId;
      }
    } else if (action[0] === 1) { // holdCurrent
      this.holdCount++;
      this.tcbStore[this.currentId!].markAsHeld();
      return this.tcbStore[this.currentId!].link;
    } else if (action[0] === 2) { // suspendCurrent
      this.tcbStore[this.currentId!].markAsSuspended();
      return this.currentId;
    } else if (action[0] === 3) { // queue
      let packet = action[1];

      var t = this.blocks[packet.id!];
      if (t == null) return t;
      this.queueCount++;
      packet.link = null;
      packet.id = this.currentId;
      return this.tcbStore[t].checkPriorityAdd(
        this.tcbStore[this.currentId!],
        packet,
      );
    }

    never(action);
  }
}

/**
 * The task is running and is currently scheduled.
 */
const STATE_RUNNING = 0;

/**
 * The task has packets left to process.
 */
const STATE_RUNNABLE = 1;

/**
 * The task is not currently running.  The task is not blocked as such and may
 * be started by the scheduler.
 */
const STATE_SUSPENDED = 2;

/**
 * The task is blocked and cannot be run until it is explicitly released.
 */
const STATE_HELD = 4;

const STATE_SUSPENDED_RUNNABLE = 3 /* TODO: STATE_SUSPENDED | STATE_RUNNABLE */;
const STATE_NOT_HELD = ~STATE_HELD;

/**
 * A task control block manages a task and the queue of work packages associated
 * with it.
 */
class TaskControlBlock {
  state: number;

  /**
   * @param {TaskControlBlock} link the preceding block in the linked block list
   * @param {int} id the id of this block
   * @param {int} priority the priority of this block
   * @param {Packet} queue the queue of packages to be processed by the task
   * @param {Task} task the task
   * @constructor
   */
  constructor(
    public link: number | null,
    public id: number,
    public priority: number,
    public queue: Packet | null,
    public task: Task,
  ) {
    if (queue == null) {
      this.state = STATE_SUSPENDED;
    } else {
      this.state = STATE_SUSPENDED_RUNNABLE;
    }
  }

  setRunning() {
    this.state = STATE_RUNNING;
  }

  markAsNotHeld() {
    this.state = this.state & STATE_NOT_HELD;
  }

  markAsHeld() {
    this.state = this.state | STATE_HELD;
  }

  isHeldOrSuspended() {
    return (this.state & STATE_HELD) != 0 ||
      (this.state == STATE_SUSPENDED);
  }

  markAsSuspended() {
    this.state = this.state | STATE_SUSPENDED;
  }

  markAsRunnable() {
    this.state = this.state | STATE_RUNNABLE;
  }

  /**
   * Runs this task, if it is ready to be run, and returns the next task to run.
   */
  run() {
    var packet;
    if (this.state == STATE_SUSPENDED_RUNNABLE) {
      packet = this.queue;
      this.queue = packet!.link;
      if (this.queue == null) {
        this.state = STATE_RUNNING;
      } else {
        this.state = STATE_RUNNABLE;
      }
    } else {
      packet = null;
    }
    return this.task.run(packet);
  }

  /**
   * Adds a packet to the worklist of this block's task, marks this as runnable if
   * necessary, and returns the next runnable object to run (the one
   * with the highest priority).
   */
  checkPriorityAdd(task: TaskControlBlock, packet: Packet) {
    if (this.queue == null) {
      this.queue = packet;
      this.markAsRunnable();
      if (this.priority > task.priority) return this.id;
    } else {
      this.queue = packet.addTo(this.queue);
    }
    return task.id;
  }

  toString() {
    return "tcb { " + this.task + "@" + this.state + " }";
  }
}

type Task = {
  run(packet: Packet | null): SchedulerAction;
  toString(): string;
};

/**
 * An idle task doesn't do any work itself but cycles control between the two
 * device tasks.
 */
class IdleTask implements Task {
  /**
   * @param {int} v1 a seed value that controls how the device tasks are scheduled
   * @param {int} count the number of times this task should be scheduled
   */
  constructor(
    public v1: number,
    public count: number,
  ) {}

  run(_packet: Packet | null) {
    this.count--;
    if (this.count == 0) return Scheduler.holdCurrent();
    if ((this.v1 & 1) == 0) {
      this.v1 = this.v1 >> 1;
      return Scheduler.release(ID_DEVICE_A);
    } else {
      this.v1 = (this.v1 >> 1) ^ 0xD008;
      return Scheduler.release(ID_DEVICE_B);
    }
  }

  toString() {
    return "IdleTask";
  }
}

/**
 * A task that suspends itself after each time it has been run to simulate
 * waiting for data from an external device.
 */
class DeviceTask implements Task {
  v1: Packet | null;

  /**
   * @constructor
   */
  constructor() {
    this.v1 = null;
  }

  run(packet: Packet | null) {
    if (packet == null) {
      if (this.v1 == null) return Scheduler.suspendCurrent();
      var v = this.v1;
      this.v1 = null;
      return Scheduler.queue(v);
    } else {
      this.v1 = packet;
      return Scheduler.holdCurrent();
    }
  }

  toString() {
    return "DeviceTask";
  }
}

/**
 * A task that manipulates work packets.
 */
class WorkerTask implements Task {
  v1: number;
  v2: number;

  /**
   * @param {int} v1 a seed used to specify how work packets are manipulated
   * @param {int} v2 another seed used to specify how work packets are manipulated
   * @constructor
   */
  constructor(v1: number, v2: number) {
    this.v1 = v1;
    this.v2 = v2;
  }

  run(packet: Packet | null) {
    if (packet == null) {
      return Scheduler.suspendCurrent();
    } else {
      if (this.v1 == ID_HANDLER_A) {
        this.v1 = ID_HANDLER_B;
      } else {
        this.v1 = ID_HANDLER_A;
      }
      packet.id = this.v1;
      packet.a1 = 0;
      for (var i = 0; i < DATA_SIZE; i++) {
        this.v2++;
        if (this.v2 > 26) this.v2 = 1;
        packet.a2[i] = this.v2;
      }
      return Scheduler.queue(packet);
    }
  }

  toString() {
    return "WorkerTask";
  }
}

/**
 * A task that manipulates work packets and then suspends itself.
 */
class HandlerTask {
  v1: Packet | null;
  v2: Packet | null;

  /**
   * @constructor
   */
  constructor() {
    this.v1 = null;
    this.v2 = null;
  }

  run(packet: Packet | null) {
    if (packet != null) {
      if (packet.kind == KIND_WORK) {
        this.v1 = packet.addTo(this.v1);
      } else {
        this.v2 = packet.addTo(this.v2);
      }
    }
    if (this.v1 != null) {
      var count = this.v1.a1;
      var v;
      if (count < DATA_SIZE) {
        if (this.v2 != null) {
          v = this.v2;
          this.v2 = this.v2.link;
          v.a1 = this.v1.a2[count]!;
          this.v1.a1 = count + 1;
          return Scheduler.queue(v);
        }
      } else {
        v = this.v1;
        this.v1 = this.v1.link;
        return Scheduler.queue(v);
      }
    }
    return Scheduler.suspendCurrent();
  }

  toString() {
    return "HandlerTask";
  }
}

const DATA_SIZE = 4;

/* --- *
 * P a c k e t
 * --- */
class Packet {
  /**
   * A simple package of data that is manipulated by the tasks.  The exact layout
   * of the payload data carried by a packet is not importaint, and neither is the
   * nature of the work performed on packets by the tasks.
   *
   * Besides carrying data, packets form linked lists and are hence used both as
   * data and worklists.
   * @param {Packet} link the tail of the linked list of packets
   * @param {int} id an ID for this packet
   * @param {int} kind the type of this packet
   * @constructor
   */

  a1 = 0;
  a2: (number | null)[] = [null, null, null, null]; // new Array(DATA_SIZE);

  constructor(
    public link: Packet | null,
    public id: number | null,
    public kind: number,
  ) {}

  /**
   * Add this packet to the end of a worklist, and return the worklist.
   * @param {Packet} queue the worklist to add this packet to
   */
  addTo(queue: Packet | null) {
    this.link = null;
    if (queue == null) return this;
    queue.link = this.addTo(queue.link);
    return queue;
  }

  toString() {
    return "Packet";
  }
}

function never(never: never): never {
  throw new Error(`Unexpected value: ${never}`);
}
