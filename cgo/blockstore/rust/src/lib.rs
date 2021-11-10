use cid::Cid;
use std::ptr;

use core::fmt;
use core2::{error, io};

use blockstore;

pub mod sys;

const ERR_NO_STORE: i32 = -1;
const ERR_NOT_FOUND: i32 = -2;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    NotFound,
    Other,
}

pub struct Blockstore {
    handle: i32,
}

impl Blockstore {
    /// Construct a new blockstore from a handle.
    pub unsafe fn new(handle: i32) -> Blockstore {
        Blockstore { handle }
    }
}

pub struct BlockReader {
    cid: Cid,
    reader: io::Cursor<Vec<u8>>,
    length: usize,
}

impl io::Read for BlockReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl blockstore::BlockReader for BlockReader {
    fn size(&self) -> usize {
        self.length
    }
    fn cid(&self) -> &Cid {
        &self.cid
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => f.write_str("not found"),
            Self::Other => f.write_str("other"),
        }
    }
}

impl error::Error for Error {}

// TODO: Implement a trait. Unfortunately, the chainsafe one is a bit tangled with the concept of a
// datastore.
impl blockstore::Blockstore for Blockstore {
    type Error = Error;
    type Block = BlockReader;

    fn has(&self, k: &Cid) -> Result<bool, Self::Error> {
        let k_bytes = k.to_bytes();
        unsafe {
            match sys::cgobs_has(self.handle, k_bytes.as_ptr(), k_bytes.len() as i32) {
                // We shouldn't get an "error not found" here, but there's no reason to be strict
                // about it.
                0 | ERR_NOT_FOUND => Ok(false),
                1 => Ok(true),
                // Panic on unknown values. There's a bug in the program.
                r @ 2.. => panic!("invalid return value from has: {}", r),
                // Panic if the store isn't registered. This means something _very_ unsafe is going
                // on and there is a bug in the program.
                ERR_NO_STORE => panic!("blockstore {} not registered", self.handle),
                // Otherwise, return "other". We should add error codes in the future.
                _ => Err(Error::Other),
            }
        }
    }

    fn get(&self, k: &Cid) -> Result<Option<Self::Block>, Self::Error> {
        let k_bytes = k.to_bytes();
        unsafe {
            let mut buf: *mut u8 = ptr::null_mut();
            let mut size: i32 = 0;
            match sys::cgobs_get(
                self.handle,
                k_bytes.as_ptr(),
                k_bytes.len() as i32,
                &mut buf,
                &mut size,
            ) {
                0 => Ok(Some(BlockReader {
                    cid: *k,
                    reader: io::Cursor::new(Vec::from_raw_parts(buf, size as usize, size as usize)),
                    length: size as usize,
                })),
                r @ 1.. => panic!("invalid return value from has: {}", r),
                ERR_NO_STORE => panic!("blockstore {} not registered", self.handle),
                ERR_NOT_FOUND => Err(Error::NotFound),
                _ => Err(Error::Other),
            }
        }
    }

    fn put(&self, k: &Cid, block: &[u8]) -> Result<(), Error> {
        let k_bytes = k.to_bytes();
        unsafe {
            match sys::cgobs_put(
                self.handle,
                k_bytes.as_ptr(),
                k_bytes.len() as i32,
                block.as_ptr(),
                block.len() as i32,
            ) {
                0 => Ok(()),
                r @ 1.. => panic!("invalid return value from has: {}", r),
                ERR_NO_STORE => panic!("blockstore {} not registered", self.handle),
                // This error makes no sense.
                ERR_NOT_FOUND => panic!("not found error on put"),
                _ => Err(Error::Other),
            }
        }
    }

    fn delete(&self, k: &Cid) -> Result<(), Error> {
        let k_bytes = k.to_bytes();
        unsafe {
            match sys::cgobs_delete(self.handle, k_bytes.as_ptr(), k_bytes.len() as i32) {
                0 => Ok(()),
                r @ 1.. => panic!("invalid return value from has: {}", r),
                ERR_NO_STORE => panic!("blockstore {} not registered", self.handle),
                // We shouldn't get this... but it's not an issue.
                ERR_NOT_FOUND => Ok(()),
                _ => Err(Error::Other),
            }
        }
    }
}
