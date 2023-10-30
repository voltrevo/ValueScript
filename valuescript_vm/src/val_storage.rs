use std::{collections::BTreeMap, io::Read, rc::Rc};

use num_bigint::BigInt;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use storage::{StorageEntity, StorageEntry, StorageEntryReader, StorageOps};

use crate::{
  vs_array::VsArray,
  vs_class::VsClass,
  vs_function::VsFunction,
  vs_object::VsObject,
  vs_storage_ptr::VsStoragePtr,
  vs_value::{ToVal, Val},
  Bytecode, VsSymbol,
};

#[derive(FromPrimitive, ToPrimitive, PartialEq, Eq)]
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
  StoragePtr,
}

impl Tag {
  fn to_byte(&self) -> u8 {
    ToPrimitive::to_u8(self).unwrap()
  }

  fn from_byte(byte: u8) -> Tag {
    FromPrimitive::from_u8(byte).unwrap()
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
  tx: &mut SO,
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
    Tag::Number => {
      let mut bytes = [0; 8];
      reader.read_exact(&mut bytes).unwrap();
      Val::Number(f64::from_le_bytes(bytes))
    }
    Tag::BigInt => {
      let len = reader.read_vlq().unwrap();
      let mut bytes = vec![0; len as usize];
      reader.read_exact(&mut bytes).unwrap();
      Val::BigInt(BigInt::from_signed_bytes_le(&bytes))
    }
    Tag::Symbol => Val::Symbol(FromPrimitive::from_u64(reader.read_vlq().unwrap()).unwrap()),
    Tag::String => {
      let len = reader.read_vlq().unwrap();
      let mut bytes = vec![0; len as usize];
      reader.read_exact(&mut bytes).unwrap();
      Val::String(String::from_utf8(bytes).unwrap().into())
    }
    Tag::Array => {
      let len = reader.read_vlq().unwrap();
      let mut items = Vec::new();

      for _ in 0..len {
        items.push(read_from_entry(tx, reader)?);
      }

      VsArray::from(items).to_val()
    }
    Tag::Object => {
      let len = reader.read_vlq().unwrap();
      let mut string_map = BTreeMap::<String, Val>::new();

      for _ in 0..len {
        let key = read_string_from_entry(tx, reader)?;
        let value = read_from_entry(tx, reader)?;

        string_map.insert(key, value);
      }

      let len = reader.read_vlq().unwrap();
      let mut symbol_map = BTreeMap::<VsSymbol, Val>::new();

      for _ in 0..len {
        let key = read_symbol_from_entry(tx, reader)?;
        let value = read_from_entry(tx, reader)?;

        symbol_map.insert(key, value);
      }

      let prototype = match read_from_entry(tx, reader)? {
        Val::Void => None,
        val => Some(val),
      };

      VsObject {
        string_map,
        symbol_map,
        prototype,
      }
      .to_val()
    }
    Tag::Function => {
      // pub bytecode: Rc<Bytecode>,
      let bytecode = read_ref_bytecode_from_entry(tx, reader)?;

      // pub meta_pos: Option<usize>,
      let meta_pos = match reader.read_u8().unwrap() {
        0 => None,
        1 => Some(reader.read_vlq().unwrap() as usize),
        _ => panic!("Invalid meta_pos byte"),
      };

      // pub is_generator: bool,
      let is_generator = match reader.read_u8().unwrap() {
        0 => false,
        1 => true,
        _ => panic!("Invalid is_generator byte"),
      };

      // pub register_count: usize,
      let register_count = reader.read_vlq().unwrap() as usize;

      // pub parameter_count: usize,
      let parameter_count = reader.read_vlq().unwrap() as usize;

      // pub start: usize,
      let start = reader.read_vlq().unwrap() as usize;

      // pub binds: Vec<Val>,
      let len = reader.read_vlq().unwrap();
      let mut binds = Vec::new();

      for _ in 0..len {
        binds.push(read_from_entry(tx, reader)?);
      }

      VsFunction {
        bytecode,
        meta_pos,
        is_generator,
        register_count,
        parameter_count,
        start,
        binds,
      }
      .to_val()
    }
    Tag::Class => {
      // pub name: String,
      let name = read_string_from_entry(tx, reader)?;

      // pub content_hash: Option<[u8; 32]>,
      let content_hash = match reader.read_u8().unwrap() {
        0 => None,
        1 => {
          let mut res = [0u8; 32];
          reader.read_exact(&mut res).unwrap();

          Some(res)
        }
        _ => panic!("Invalid content_hash byte"),
      };

      // pub constructor: Val,
      let constructor = read_from_entry(tx, reader)?;

      // pub prototype: Val,
      let prototype = read_from_entry(tx, reader)?;

      // pub static_: Val,
      let static_ = read_from_entry(tx, reader)?;

      VsClass {
        name,
        content_hash,
        constructor,
        prototype,
        static_,
      }
      .to_val()
    }
    Tag::Static => todo!(),
    Tag::Dynamic => todo!(),
    Tag::CopyCounter => todo!(),
    Tag::StoragePtr => VsStoragePtr::from_ptr(reader.read_ref().unwrap()).to_val(),
  })
}

fn read_string_from_entry<E, SO: StorageOps<E>>(
  tx: &mut SO,
  reader: &mut StorageEntryReader,
) -> Result<String, E> {
  let len = reader.read_vlq().unwrap();
  let mut bytes = vec![0; len as usize];
  reader.read_exact(&mut bytes).unwrap();
  Ok(String::from_utf8(bytes).unwrap())
}

fn read_symbol_from_entry<E, SO: StorageOps<E>>(
  _tx: &mut SO,
  reader: &mut StorageEntryReader,
) -> Result<VsSymbol, E> {
  Ok(FromPrimitive::from_u64(reader.read_vlq().unwrap()).unwrap())
}

fn read_ref_bytecode_from_entry<E, SO: StorageOps<E>>(
  tx: &mut SO,
  reader: &mut StorageEntryReader,
) -> Result<Rc<Bytecode>, E> {
  let ptr = reader.read_ref().unwrap();
  let entry = tx.read(ptr)?.unwrap();

  // TODO: Cached reads
  Ok(Rc::new(Bytecode::new(entry.data)))
}
