export default function main() {
  let idGen = new IdGenerator();

  return [
    idGen.generate(),
    idGen.generate(),
    idGen.generate(),
  ];
}

class IdGenerator {
  nextId: number;

  constructor() {
    this.nextId = 1;
  }

  generate() {
    const result = this.nextId;
    this.nextId++;

    return result;
  }
}