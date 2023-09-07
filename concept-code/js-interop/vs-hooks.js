// A ValueScript sandbox similar to QuickJS Emscripten would be valuable.
// https://github.com/justjake/quickjs-emscripten

import { useVS } from "value-script";

// an object containing any settings to customise valuescript
// e.g. max cycles, variables exposed to it, external functionality, etc.
const context = {
  maxCycles: 1000,
  memoryLimitBytes: 1024 * 1024,
  // arbitrary nested document tree (with functions) which is persisted across calls.
  data: {
    maxBlocks: 10,
    request: async (config) => {
      await axios(config);
    },
    sleep: async (ms) => {
      await new Promise((resolve) => setTimeout(resolve, ms));
    },
    console: {
      log: (...args) => {
        console.log(...args);
      },
    },
  },
};

const Page = () => {
  // calling setContext should allow you to update the shared context.
  // isReady is used when getting WASM ready and the initial context.
  const { vs, isReady, setContext } = useVS(context);

  const handleExecute = async () => {
    try {
      const disposableData = {
        blocks: [],
        input: {},
        // etc
      };

      // should be able to access any of the methods exposed in the two contexts.
      const code = `const a = 50
        const b = 25
        
        a * b / 2`;

      const result = await vs.execute(disposableData, code);

      // do what you want with the result.
    } catch (error) {
      // handle error
    }
  };

  return <button onClick={handleExecute}>Execute</button>;
};

export default Page;
