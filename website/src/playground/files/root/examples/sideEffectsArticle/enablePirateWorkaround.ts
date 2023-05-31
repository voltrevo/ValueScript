export default function main() {
  let pirateEnabled = false;

  function greet(pirateEnabled: boolean) {
    if (!pirateEnabled) {
      return "Hi";
    }

    return "Ahoy";
  }

  function enablePirate(pirateEnabled: boolean): [boolean, string] {
    pirateEnabled = true;
    return [pirateEnabled, "Done"];
  }

  const greetResponse1 = greet(pirateEnabled);

  let enablePirateResponse: string;
  [pirateEnabled, enablePirateResponse] = enablePirate(pirateEnabled);

  const greetResponse2 = greet(pirateEnabled);

  return [greetResponse1, enablePirateResponse, greetResponse2];
}
