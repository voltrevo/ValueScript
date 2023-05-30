// test_output! ["🚀","🍹","a","b","c","£","한","🎨"]

export default function () {
  const str = "🚀🍹abc£한🎨";
  let outputs = [];

  for (const c of str) {
    outputs.push(c);
  }

  return outputs;
}
