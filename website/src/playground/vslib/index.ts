/* eslint-disable @typescript-eslint/no-explicit-any */

export async function initVslib() {
  const wasm: Record<string, any> = (await WebAssembly.instantiateStreaming(
    fetch(`${location.origin}/value_script_bg.wasm`),
    {
      './valuescript_wasm_bg.js': {
        __wbindgen_throw,
        __wbindgen_string_new,
        __wbindgen_object_drop_ref,
        __wbindgen_string_get,
        __wbg_call_9495de66fdbe016b,
        __wbg_jsgeterrormessage_11e1f21ab8a95c33,
        __wbg_jsconsolelog_64bb2dc407556512,
      },
    },
  )).instance.exports;

  let WASM_VECTOR_LEN = 0;

  let cachegetUint8Memory0: Uint8Array | null = null;
  function getUint8Memory0() {
    if (
      cachegetUint8Memory0 === null ||
      cachegetUint8Memory0.buffer !== wasm.memory.buffer
    ) {
      cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
  }

  const cachedTextEncoder = new TextEncoder();

  const encodeString = function (arg: string, view: Uint8Array) {
    return cachedTextEncoder.encodeInto(arg, view);
  };

  function passStringToWasm0(
    arg: string,
    malloc: (len: number) => number,
    realloc: (a: number, b: number, c: number) => number,
  ) {
    if (realloc === undefined) {
      const buf = cachedTextEncoder.encode(arg);
      const ptr = malloc(buf.length);
      getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
      WASM_VECTOR_LEN = buf.length;
      return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len);

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
      const code = arg.charCodeAt(offset);
      if (code > 0x7F) break;
      mem[ptr + offset] = code;
    }

    if (offset !== len) {
      if (offset !== 0) {
        arg = arg.slice(offset);
      }
      ptr = realloc(ptr, len, len = offset + arg.length * 3);
      const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
      const ret = encodeString(arg, view);

      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      offset += ret.written!;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
  }

  let cachegetInt32Memory0: Int32Array | null = null;
  function getInt32Memory0() {
    if (
      cachegetInt32Memory0 === null ||
      cachegetInt32Memory0.buffer !== wasm.memory.buffer
    ) {
      cachegetInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory0;
  }

  const cachedTextDecoder = new TextDecoder('utf-8', {
    ignoreBOM: true,
    fatal: true,
  });

  cachedTextDecoder.decode();

  function getStringFromWasm0(ptr: number, len: number) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
  }

  function compile(entry_point: string, read_file: (path: string) => string) {
    let r0, r1;

    try {
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      const ptr0 = passStringToWasm0(entry_point, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.compile(retptr, ptr0, len0, addBorrowedObject(read_file));
      r0 = getInt32Memory0()[retptr / 4 + 0];
      r1 = getInt32Memory0()[retptr / 4 + 1];
      return getStringFromWasm0(r0, r1);
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
      heap[stack_pointer++] = undefined;
      wasm.__wbindgen_free(r0, r1);
    }
  }

  function run(source: string) {
    let r0, r1;

    try {
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      const ptr0 = passStringToWasm0(
        source,
        wasm.__wbindgen_malloc,
        wasm.__wbindgen_realloc,
      );
      const len0 = WASM_VECTOR_LEN;
      wasm.run(retptr, ptr0, len0);
      r0 = getInt32Memory0()[retptr / 4 + 0];
      r1 = getInt32Memory0()[retptr / 4 + 1];
      return getStringFromWasm0(r0, r1);
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
      wasm.__wbindgen_free(r0, r1);
    }
  }

  function run_linked(entry_point: string, read_file: (path: string) => string) {
    let r0 = undefined;
    let r1 = undefined;

    try {
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      const ptr0 = passStringToWasm0(entry_point, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len0 = WASM_VECTOR_LEN;
      wasm.run_linked(retptr, ptr0, len0, addBorrowedObject(read_file));
      r0 = getInt32Memory0()[retptr / 4 + 0];
      r1 = getInt32Memory0()[retptr / 4 + 1];
      return getStringFromWasm0(r0, r1);
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
      heap[stack_pointer++] = undefined;
      wasm.__wbindgen_free(r0, r1);
    }
  }

  let stack_pointer = 128;

  function addBorrowedObject(obj: unknown) {
    if (stack_pointer == 1) throw new Error('out of js stack');
    heap[--stack_pointer] = obj;
    return stack_pointer;
  }

  function __wbindgen_throw(arg0: number, arg1: number) {
    throw new Error(getStringFromWasm0(arg0, arg1));
  }

  function __wbg_jsgeterrormessage_11e1f21ab8a95c33(arg0: number, arg1: number) {
    const ret = js_get_error_message(getObject(arg1));
    const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
  }

  function js_get_error_message(e: Error) {
    return e.message;
  }

  function getObject(idx: number) {
    return heap[idx];
  }

  const heap = new Array(128).fill(undefined);

  heap.push(undefined, null, true, false);

  let heap_next = heap.length;

  function __wbindgen_string_new(arg0: number, arg1: number) {
    const ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
  }

  function addHeapObject(obj: unknown) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
  }

  function __wbindgen_object_drop_ref(arg0: number) {
    takeObject(arg0);
  }

  function takeObject(idx: number) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
  }

  function dropObject(idx: number) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
  }

  function __wbindgen_string_get(arg0: number, arg1: number) {
    const obj = getObject(arg1);
    const ret = typeof (obj) === 'string' ? obj : undefined;
    const ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      ret!,
      wasm.__wbindgen_malloc,
      wasm.__wbindgen_realloc,
    );
    const len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
  }

  function isLikeNone(x: unknown) {
    return x === undefined || x === null;
  }

  function __wbg_call_9495de66fdbe016b() {
    return handleError(function (arg0: number, arg1: number, arg2: number) {
      const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
      return addHeapObject(ret);
      // eslint-disable-next-line prefer-rest-params
    }, arguments as any);
  }

  function handleError(this: any, f: any, args: unknown[]) {
    try {
      return f.apply(this, args);
    } catch (e) {
      wasm.__wbindgen_exn_store(addHeapObject(e));
    }
  }

  function __wbg_jsconsolelog_64bb2dc407556512(arg0: number, arg1: number) {
    js_console_log(getStringFromWasm0(arg0, arg1));
  }

  function js_console_log(arg0: string) {
    console.log(arg0);
  }

  return {
    compile,
    run,
    runLinked: run_linked,
  };
}
