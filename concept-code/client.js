import { Database } from 'value-script';

const db = await Database.connect('localhost:3000/example');

console.log(await db.exec(function() {
  return 1 + 1;
})); // 2

console.log(await db.exec(function() {
  let x = [];
  x.push(x);
  return x;
})); // [[]]

console.log(await db.exec(function() {
  return this;
})); // undefined

console.log(await db.exec(function({ assign }) {
  assign(this, {});
})); // undefined

console.log(await db.exec(function() {
  return this;
})); // {}

console.log(await db.exec(function() {
  this.some = 'data';
})); // undefined

console.log(await db.exec(function() {
  return this.some;
})); // 'data'

console.log(await db.exec(function() {
  this.more = 'other data';
})); // undefined

console.log(await db.exec(function() {
  return this.some.length + this.more.length;
})); // 14

console.log(await db.exec(function() {
  return { result: this };
})); // { result: { some: 'data', more: 'other data' } }

// above tx retrieves the entire db, which would fail on a large db
// instead, execHook returns another db-type object for further queries

let subDb = await db.execHook(function() {
  return { result: this };
});

console.log(await subDb.exec(function() {
  return this.result.some;
})); // 'data'

try {
  console.log(await db.exec(function() {
    return () => {
      let x = [];
      x.push(x);
      return x;
    };
  }));
} catch (error) {
  console.log(error); // Error: Failed to return function to js
}

subDb = await db.execHook(function() {
  return () => {
    let x = [];
    x.push(x);
    return x;
  };
});

console.log(await subDb.exec(function() {
  return this();
})); // [[]]
