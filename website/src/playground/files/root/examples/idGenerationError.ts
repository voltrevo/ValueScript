export default function main() {
  let nextId = 1;
      
  function generateId() {
    const result = nextId;
    nextId++;
  
    return result;
  }

  return [
    generateId(),
    generateId(),
    generateId(),
  ];
}