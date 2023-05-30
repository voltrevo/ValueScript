//! test_output([undefined,"🚀","","","","🍹","","","","a","b","c","£","","한","","","🎨","","","",undefined])

export default function () {
  const str = "🚀🍹abc£한🎨";
  let outputs = [];

  for (let i = -1; i <= str.length; i++) {
    outputs.push(str[i]);
  }

  return outputs;
}
