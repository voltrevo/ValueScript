use std::{collections::BTreeMap, error::Error, rc::Rc};

use num_bigint::BigInt;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use storage::{
  RcKey, StorageBackend, StorageEntity, StorageEntry, StorageEntryReader, StorageEntryWriter,
  StorageError, StorageTx,
};

use crate::{
  vs_class::VsClass,
  vs_function::VsFunction,
  vs_object::VsObject,
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

  fn from_byte(byte: u8) -> Result<Tag, Box<dyn Error>> {
    FromPrimitive::from_u8(byte).ok_or("Invalid tag byte".into())
  }
}

impl<'a, SB: StorageBackend, Tx: StorageTx<'a, SB>> StorageEntity<'a, SB, Tx> for Val {
  fn from_storage_entry(tx: &mut Tx, entry: StorageEntry) -> Result<Self, StorageError<SB>> {
    let mut reader = StorageEntryReader::new(&entry);
    let res = read_from_entry(tx, &mut reader);
    assert!(reader.done());

    res
  }

  fn to_storage_entry(&self, tx: &mut Tx) -> Result<StorageEntry, StorageError<SB>> {
    let mut entry = StorageEntry {
      ref_count: 1,
      refs: vec![],
      data: vec![],
    };

    let writer = &mut StorageEntryWriter::new(&mut entry);

    match self {
      Val::Array(a) => {
        writer.write_u8(Tag::Array.to_byte());
        writer.write_vlq(a.elements.len());

        for item in a.elements.iter() {
          write_to_entry(item, tx, writer)?;
        }
      }
      Val::Object(obj) => {
        writer.write_u8(Tag::Object.to_byte());

        writer.write_vlq(obj.string_map.len());

        for (key, value) in obj.string_map.iter() {
          let key_bytes = key.as_bytes();
          writer.write_vlq(key_bytes.len());
          writer.write_bytes(key_bytes);
          write_to_entry(value, tx, writer)?;
        }

        writer.write_vlq(obj.symbol_map.len());

        for (key, value) in obj.symbol_map.iter() {
          writer.write_vlq(key.to_usize().unwrap());
          write_to_entry(value, tx, writer)?;
        }

        match &obj.prototype {
          None => writer.write_u8(0),
          Some(val) => {
            writer.write_u8(1);
            write_to_entry(val, tx, writer)?
          }
        };
      }
      Val::Function(f) => {
        let VsFunction {
          bytecode,
          meta_pos,
          is_generator,
          register_count,
          parameter_count,
          start,
          binds,
        } = f.as_ref();

        writer.write_u8(Tag::Function.to_byte());

        write_ref_bytecode_to_entry(tx, writer, bytecode)?;

        match *meta_pos {
          None => writer.write_u8(0),
          Some(pos) => {
            writer.write_u8(1);
            writer.write_vlq(pos);
          }
        };

        writer.write_u8(if *is_generator { 1 } else { 0 });
        writer.write_vlq(*register_count);
        writer.write_vlq(*parameter_count);
        writer.write_vlq(*start);
        writer.write_vlq(binds.len());

        for bind in binds.iter() {
          write_to_entry(bind, tx, writer)?;
        }
      }

      Val::Void
      | Val::Undefined
      | Val::Null
      | Val::Bool(_)
      | Val::Number(_)
      | Val::BigInt(_)
      | Val::Symbol(_)
      | Val::String(_)
      | Val::Class(_)
      | Val::Static(_)
      | Val::Dynamic(_)
      | Val::CopyCounter(_)
      | Val::StoragePtr(_) => {
        write_to_entry(self, tx, writer)?;
      }
    };

    Ok(entry)
  }
}

fn write_to_entry<'a, SB: StorageBackend, Tx: StorageTx<'a, SB>>(
  val: &Val,
  tx: &mut Tx,
  writer: &mut StorageEntryWriter,
) -> Result<(), StorageError<SB>> {
  match val {
    Val::Void => {
      writer.write_u8(Tag::Void.to_byte());
    }
    Val::Undefined => {
      writer.write_u8(Tag::Undefined.to_byte());
    }
    Val::Null => {
      writer.write_u8(Tag::Null.to_byte());
    }
    Val::Bool(b) => {
      writer.write_u8(Tag::Bool.to_byte());
      writer.write_u8(if *b { 1 } else { 0 });
    }
    Val::Number(n) => {
      writer.write_u8(Tag::Number.to_byte());
      writer.write_bytes(&n.to_le_bytes());
    }
    Val::BigInt(b) => {
      writer.write_u8(Tag::BigInt.to_byte());
      let bytes = b.to_signed_bytes_le();
      writer.write_vlq(bytes.len());
      writer.write_bytes(&bytes);
    }
    Val::Symbol(s) => {
      writer.write_u8(Tag::Symbol.to_byte());
      writer.write_vlq(s.to_usize().unwrap());
    }
    Val::String(s) => {
      writer.write_u8(Tag::String.to_byte());
      let bytes = s.as_bytes();
      writer.write_vlq(bytes.len());
      writer.write_bytes(bytes);
    }
    Val::Array(a) => write_ptr_to_entry(tx, writer, RcKey::from(a.clone()), val)?,
    Val::Object(obj) => write_ptr_to_entry(tx, writer, RcKey::from(obj.clone()), val)?,
    Val::Function(f) => write_ptr_to_entry(tx, writer, RcKey::from(f.clone()), val)?,
    Val::Class(c) => {
      writer.write_u8(Tag::Class.to_byte());

      let VsClass {
        name,
        content_hash,
        constructor,
        prototype,
        static_,
      } = c.as_ref();

      let name_bytes = name.as_bytes();
      writer.write_vlq(name_bytes.len());
      writer.write_bytes(name_bytes);

      match *content_hash {
        None => writer.write_u8(0),
        Some(hash) => {
          writer.write_u8(1);
          writer.write_bytes(&hash);
        }
      };

      write_to_entry(constructor, tx, writer)?;
      write_to_entry(prototype, tx, writer)?;
      write_to_entry(static_, tx, writer)?;
    }
    Val::Static(_s) => {
      writer.write_u8(Tag::Static.to_byte());
      todo!()
    }
    Val::Dynamic(_d) => {
      writer.write_u8(Tag::Dynamic.to_byte());
      todo!()
    }
    Val::CopyCounter(_cc) => {
      writer.write_u8(Tag::CopyCounter.to_byte());
      todo!()
    }
    Val::StoragePtr(ptr) => {
      writer.write_u8(Tag::StoragePtr.to_byte());
      writer.entry.refs.push(ptr.ptr);
    }
  };

  Ok(())
}

fn write_ptr_to_entry<'a, SB: StorageBackend, Tx: StorageTx<'a, SB>>(
  tx: &mut Tx,
  writer: &mut StorageEntryWriter,
  key: RcKey,
  val: &Val,
) -> Result<(), StorageError<SB>> {
  if let Some(ptr) = tx.cache_get(key.clone()) {
    writer.write_u8(Tag::StoragePtr.to_byte());
    writer.entry.refs.push(ptr);
  } else {
    let ptr = tx.store_and_cache(val, key)?;
    writer.write_u8(Tag::StoragePtr.to_byte());
    writer.entry.refs.push(ptr);
  }

  Ok(())
}

fn read_from_entry<'a, SB: StorageBackend, Tx: StorageTx<'a, SB>>(
  tx: &mut Tx,
  reader: &mut StorageEntryReader,
) -> Result<Val, StorageError<SB>> {
  let tag = Tag::from_byte(reader.read_u8()?)?;

  Ok(match tag {
    Tag::Void => Val::Void,
    Tag::Undefined => Val::Undefined,
    Tag::Null => Val::Null,
    Tag::Bool => Val::Bool(match reader.read_u8()? {
      0 => false,
      1 => true,
      _ => panic!("Invalid bool byte"),
    }),
    Tag::Number => Val::Number(reader.read_u8_array().map(f64::from_le_bytes)?),
    Tag::BigInt => BigInt::from_signed_bytes_le(&reader.read_vlq_buf()?).to_val(),
    Tag::Symbol => Val::Symbol(FromPrimitive::from_usize(reader.read_vlq()?).unwrap()),
    Tag::String => String::from_utf8(reader.read_vlq_buf()?)
      .map_err(StorageError::from)?
      .to_val(),
    Tag::Array => {
      let len = reader.read_vlq()?;
      let mut items = Vec::new();

      for _ in 0..len {
        items.push(read_from_entry(tx, reader)?);
      }

      items.to_val()
    }
    Tag::Object => {
      let len = reader.read_vlq()?;
      let mut string_map = BTreeMap::<String, Val>::new();

      for _ in 0..len {
        let key = read_string_from_entry(reader)?;
        let value = read_from_entry(tx, reader)?;

        string_map.insert(key, value);
      }

      let len = reader.read_vlq()?;
      let mut symbol_map = BTreeMap::<VsSymbol, Val>::new();

      for _ in 0..len {
        let key = read_symbol_from_entry(reader)?;
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
      let bytecode = read_ref_bytecode_from_entry(tx, reader)?;

      let meta_pos = match reader.read_u8()? {
        0 => None,
        1 => Some(reader.read_vlq()?),
        _ => panic!("Invalid meta_pos byte"),
      };

      let is_generator = match reader.read_u8()? {
        0 => false,
        1 => true,
        _ => panic!("Invalid is_generator byte"),
      };

      let register_count = reader.read_vlq()?;
      let parameter_count = reader.read_vlq()?;
      let start = reader.read_vlq()?;

      let len = reader.read_vlq()?;
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
      let name = read_string_from_entry(reader)?;

      let content_hash = match reader.read_u8()? {
        0 => None,
        1 => Some(reader.read_u8_array()?),
        _ => panic!("Invalid content_hash byte"),
      };

      let constructor = read_from_entry(tx, reader)?;
      let prototype = read_from_entry(tx, reader)?;
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
    Tag::StoragePtr => reader.read_ref()?.to_val(),
  })
}

fn read_string_from_entry(reader: &mut StorageEntryReader) -> Result<String, Box<dyn Error>> {
  Ok(String::from_utf8(reader.read_vlq_buf()?)?)
}

fn read_symbol_from_entry(reader: &mut StorageEntryReader) -> Result<VsSymbol, Box<dyn Error>> {
  Ok(FromPrimitive::from_usize(reader.read_vlq()?).unwrap())
}

fn read_ref_bytecode_from_entry<'a, SB: StorageBackend, Tx: StorageTx<'a, SB>>(
  tx: &mut Tx,
  reader: &mut StorageEntryReader,
) -> Result<Rc<Bytecode>, StorageError<SB>> {
  let ptr = reader.read_ref()?;
  let entry = tx.read_or_err(ptr)?;

  // TODO: Cached reads
  Ok(Rc::new(Bytecode::from_storage_entry(tx, entry)?))
}

fn write_ref_bytecode_to_entry<'a, SB: StorageBackend, Tx: StorageTx<'a, SB>>(
  tx: &mut Tx,
  writer: &mut StorageEntryWriter,
  bytecode: &Rc<Bytecode>,
) -> Result<(), StorageError<SB>> {
  let key = RcKey::from(bytecode.clone());

  if let Some(ptr) = tx.cache_get(key.clone()) {
    writer.entry.refs.push(ptr);
  } else {
    let ptr = tx.store_and_cache(bytecode.as_ref(), key)?;
    writer.entry.refs.push(ptr);
  }

  Ok(())
}
