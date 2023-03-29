import assert from '../helpers/assert';

const entries = Object.entries(
  import.meta.glob(
    './root/**/*.ts',
    { as: 'raw' },
  ),
);

export default Object.fromEntries(await Promise.all(
  entries.map(async ([path, module]) => {
    const prefix = './root';
    assert(path.startsWith(prefix));
    return [path.slice('./root'.length), (await module()).trim()];
  }),
));
