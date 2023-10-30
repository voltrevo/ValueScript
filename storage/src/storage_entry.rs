use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::storage_ptr::StorageEntryPtr;

#[derive(Serialize, Deserialize)]
pub struct StorageEntry {
  pub(crate) ref_count: u64,
  pub refs: Vec<StorageEntryPtr>,
  pub data: Vec<u8>,
}

pub struct StorageEntryReader<'a> {
  pub entry: &'a StorageEntry,
  pub refs_i: usize,
  pub data_i: usize,
}

impl<'a> StorageEntryReader<'a> {
  pub fn new(entry: &'a StorageEntry) -> Self {
    Self {
      entry,
      refs_i: 0,
      data_i: 0,
    }
  }

  pub fn read_ref(&mut self) -> std::io::Result<StorageEntryPtr> {
    if self.refs_i >= self.entry.refs.len() {
      return Err(eof());
    }

    let ptr = self.entry.refs[self.refs_i];
    self.refs_i += 1;
    Ok(ptr)
  }

  pub fn read_u8(&mut self) -> std::io::Result<u8> {
    if self.data_i >= self.entry.data.len() {
      return Err(eof());
    }

    let byte = self.entry.data[self.data_i];
    self.data_i += 1;
    Ok(byte)
  }

  pub fn peek_u8(&self) -> std::io::Result<u8> {
    if self.data_i >= self.entry.data.len() {
      return Err(eof());
    }

    Ok(self.entry.data[self.data_i])
  }

  pub fn read_u64(&mut self) -> std::io::Result<u64> {
    let mut bytes = [0; 8];
    self.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
  }

  pub fn read_vlq(&mut self) -> std::io::Result<u64> {
    let mut result = 0;
    let mut shift = 0;

    loop {
      let byte = self.read_u8()?;

      result |= ((byte & 0x7f) as u64) << shift;

      if byte & 0x80 == 0 {
        break;
      }

      shift += 7;
    }

    Ok(result)
  }
}

impl Read for StorageEntryReader<'_> {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    let bytes = self
      .entry
      .data
      .get(self.data_i..self.data_i + buf.len())
      .ok_or(eof())?;

    buf.copy_from_slice(bytes);
    self.data_i += buf.len();

    Ok(buf.len())
  }
}

fn eof() -> std::io::Error {
  std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "not enough bytes")
}
