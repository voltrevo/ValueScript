use storage::{StorageEntity, StorageEntry, StorageEntryReader, StorageOps};

use crate::vs_value::Val;

enum Tag {
  Void,
  Undefined,
  Null,
  Bool,
  Number,
  BigInt,
  Symbol,
  String,
  Array,
  Object,
  Function,
  Class,
  Static,
  Dynamic,
  CopyCounter,
}

impl Tag {
  fn to_byte(&self) -> u8 {
    match self {
      Tag::Void => 0,
      Tag::Undefined => 1,
      Tag::Null => 2,
      Tag::Bool => 3,
      Tag::Number => 4,
      Tag::BigInt => 5,
      Tag::Symbol => 6,
      Tag::String => 7,
      Tag::Array => 8,
      Tag::Object => 9,
      Tag::Function => 10,
      Tag::Class => 11,
      Tag::Static => 12,
      Tag::Dynamic => 13,
      Tag::CopyCounter => 14,
    }
  }

  fn from_byte(byte: u8) -> Self {
    match byte {
      0 => Tag::Void,
      1 => Tag::Undefined,
      2 => Tag::Null,
      3 => Tag::Bool,
      4 => Tag::Number,
      5 => Tag::BigInt,
      6 => Tag::Symbol,
      7 => Tag::String,
      8 => Tag::Array,
      9 => Tag::Object,
      10 => Tag::Function,
      11 => Tag::Class,
      12 => Tag::Static,
      13 => Tag::Dynamic,
      14 => Tag::CopyCounter,
      _ => panic!("Invalid tag byte: {}", byte),
    }
  }
}

impl StorageEntity for Val {
  fn from_storage_entry<E, Tx: StorageOps<E>>(tx: &mut Tx, entry: StorageEntry) -> Result<Self, E> {
    read_from_entry(tx, &mut StorageEntryReader::new(&entry))
    // TODO: assert that we've read the whole entry
  }

  fn to_storage_entry<E, Tx: StorageOps<E>>(&self, tx: &mut Tx) -> Result<StorageEntry, E> {
    todo!()
  }
}

fn write_to_entry<E, SO: StorageOps<E>>(
  val: &Val,
  tx: &mut SO,
  entry: &mut StorageEntry,
) -> Result<(), E> {
  match val {
    Val::Void => {
      entry.data.push(Tag::Void.to_byte());
    }
    Val::Undefined => {
      entry.data.push(Tag::Undefined.to_byte());
    }
    Val::Null => {
      entry.data.push(Tag::Null.to_byte());
    }
    Val::Bool(b) => {
      entry.data.push(Tag::Bool.to_byte());
      entry.data.push(if *b { 1 } else { 0 });
    }
    Val::Number(n) => {
      entry.data.push(Tag::Number.to_byte());
      entry.data.extend_from_slice(&n.to_le_bytes());
    }
    Val::BigInt(b) => {
      entry.data.push(Tag::BigInt.to_byte());
      todo!()
    }
    Val::Symbol(s) => {
      entry.data.push(Tag::Symbol.to_byte());
      todo!()
    }
    Val::String(s) => {
      entry.data.push(Tag::String.to_byte());
      todo!()
    }
    Val::Array(a) => {
      entry.data.push(Tag::Array.to_byte());
      todo!()
    }
    Val::Object(o) => {
      entry.data.push(Tag::Object.to_byte());
      todo!()
    }
    Val::Function(f) => {
      entry.data.push(Tag::Function.to_byte());
      todo!()
    }
    Val::Class(c) => {
      entry.data.push(Tag::Class.to_byte());
      todo!()
    }
    Val::Static(s) => {
      entry.data.push(Tag::Static.to_byte());
      todo!()
    }
    Val::Dynamic(d) => {
      entry.data.push(Tag::Dynamic.to_byte());
      todo!()
    }
    Val::CopyCounter(cc) => {
      entry.data.push(Tag::CopyCounter.to_byte());
      todo!()
    }
    Val::StoragePtr(ptr) => {
      entry.data.push(Tag::CopyCounter.to_byte());
      todo!()
    }
  };

  Ok(())
}

fn read_from_entry<E, SO: StorageOps<E>>(
  _tx: &mut SO,
  reader: &mut StorageEntryReader,
) -> Result<Val, E> {
  let tag = Tag::from_byte(reader.read_u8().unwrap());

  Ok(match tag {
    Tag::Void => Val::Void,
    Tag::Undefined => Val::Undefined,
    Tag::Null => Val::Null,
    Tag::Bool => Val::Bool(match reader.read_u8().unwrap() {
      0 => false,
      1 => true,
      _ => panic!("Invalid bool byte"),
    }),
    Tag::Number => todo!(),
    Tag::BigInt => todo!(),
    Tag::Symbol => todo!(),
    Tag::String => todo!(),
    Tag::Array => todo!(),
    Tag::Object => todo!(),
    Tag::Function => todo!(),
    Tag::Class => todo!(),
    Tag::Static => todo!(),
    Tag::Dynamic => todo!(),
    Tag::CopyCounter => todo!(),
  })
}
