use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::storage_ptr::StorageEntryPtr;

#[derive(Serialize, Deserialize)]
pub struct StorageEntry {
  pub ref_count: u64,
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

  pub fn done(&self) -> bool {
    self.refs_i == self.entry.refs.len() && self.data_i == self.entry.data.len()
  }

  pub fn read_u8_array<const L: usize>(&mut self) -> Result<[u8; L], Box<dyn Error>> {
    if self.data_i + L > self.entry.data.len() {
      return Err(eof().into());
    }

    let mut bytes = [0; L];
    bytes.copy_from_slice(&self.entry.data[self.data_i..self.data_i + L]);

    self.data_i += L;

    Ok(bytes)
  }

  pub fn read_buf(&mut self, len: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    if self.data_i + len > self.entry.data.len() {
      return Err(eof().into());
    }

    let buf = self.entry.data[self.data_i..self.data_i + len].to_vec();
    self.data_i += len;

    Ok(buf)
  }

  pub fn read_vlq_buf(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
    let len = self.read_vlq()?;
    self.read_buf(len)
  }

  pub fn read_ref(&mut self) -> Result<StorageEntryPtr, Box<dyn Error>> {
    if self.refs_i >= self.entry.refs.len() {
      return Err(eof().into());
    }

    let ptr = self.entry.refs[self.refs_i];
    self.refs_i += 1;
    Ok(ptr)
  }

  pub fn read_u8(&mut self) -> Result<u8, Box<dyn Error>> {
    if self.data_i >= self.entry.data.len() {
      return Err(eof().into());
    }

    let byte = self.entry.data[self.data_i];
    self.data_i += 1;
    Ok(byte)
  }

  pub fn peek_u8(&self) -> Result<u8, Box<dyn Error>> {
    if self.data_i >= self.entry.data.len() {
      return Err(eof().into());
    }

    Ok(self.entry.data[self.data_i])
  }

  pub fn read_u64(&mut self) -> Result<u64, Box<dyn Error>> {
    self.read_u8_array().map(u64::from_le_bytes)
  }

  pub fn read_vlq(&mut self) -> Result<usize, Box<dyn Error>> {
    let mut result = 0;
    let mut shift = 0;

    loop {
      let byte = self.read_u8()?;

      result |= ((byte & 0x7f) as usize) << shift;

      if byte & 0x80 == 0 {
        break;
      }

      shift += 7;
    }

    Ok(result)
  }
}

fn eof() -> std::io::Error {
  std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "not enough bytes")
}

pub struct StorageEntryWriter<'a> {
  pub entry: &'a mut StorageEntry,
}

impl<'a> StorageEntryWriter<'a> {
  pub fn new(entry: &'a mut StorageEntry) -> Self {
    Self { entry }
  }

  pub fn write_u8(&mut self, byte: u8) {
    self.entry.data.push(byte);
  }

  pub fn write_bytes(&mut self, bytes: &[u8]) {
    self.entry.data.extend_from_slice(bytes);
  }

  pub fn write_vlq(&mut self, mut num: usize) {
    loop {
      let mut byte = (num & 0x7f) as u8;

      if num != 0 {
        byte |= 0x80;
      }

      self.write_u8(byte);

      if num == 0 {
        break;
      }

      num >>= 7;
    }
  }

  pub fn write_vlq_buf(&mut self, buf: &[u8]) {
    self.write_vlq(buf.len());
    self.write_bytes(buf);
  }
}
