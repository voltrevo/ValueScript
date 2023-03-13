// test_output! ["🚀","","","","🍹","","","","a","b","c","£","","한","","","🎨","","",""]
// This is wrong. It should be: ["🚀","🍹","a","b","c","£","한","🎨"].
// The reason is that for-of is currently approximated using indexing from 0 to .length. This is
// expected to be fixed when iterators are added to the language.

export default function () {
  const str = "🚀🍹abc£한🎨";
  let outputs = [];

  for (const c of str) {
    outputs.push(c);
  }

  return outputs;
}
