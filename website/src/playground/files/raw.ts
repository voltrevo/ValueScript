import assert from "../helpers/assert.ts";

const entries = Object.entries(
  import.meta.glob("./root/**/*.*", { as: "raw" }),
);

export default Object.fromEntries(
  await Promise.all(
    entries.map(async ([path, module]) => {
      const prefix = "./root";
      assert(path.startsWith(prefix));
      return [path.slice("./root".length), (await module()).trim()];
    }),
  ),
);
