//! This crate provides support for compressing files using Sega's CMP
//! run-length encoding scheme, which was used in games for the
//! Sega Saturn game console.
//!
//! [Run-length encoding](https://en.wikipedia.org/wiki/Run-length_encoding)
//! is a simple kind of compression which works by analyzing data for repetitive
//! sequences of data (or "runs"); any repeating sequences can be replaced by
//! commands to repeat X data N times.
//!
//! For example, imagine the text `AAAAAAAABBBBAABA`.
//! If you described it in terms of patterns, you could think of it as being
//! `8A4B2A1B1A` - which takes up 10 bytes instead of 16.
//!
//! Sega included CMP in the official Saturn SDK; they provided a closed-source
//! commandline tool to compress data and source code to do compression from
//! within games. As a result, many games used it for data with lots of repetitive
//! content such as text and graphics.
//! This crate's CMP implementation aims to be 100% compatible with the original
//! decoder; it produces identical compressed data to the original encoder.
//! Its output has been tested to work in commercial Saturn games which use CMP.
//!
//! This crate provides two basic functions: the header-generating `create_header`,
//! and the data-creating `compress`. Most Saturn games store both in the same place,
//! with the header followed immedaitely by the compressed data.

use std::error::Error;
use std::fmt;
use std::slice;
use std::ptr;

extern crate libc;
use libc::{c_int, intptr_t};

/// Used to denote the width of data to compress.
/// Because CMP compression was created to be used on the SH-2 CPU, the size names
/// come from the three sizes of data used on the SH-2.
#[derive(Copy, Clone)]
pub enum Size {
    /// 8-bit
    Byte,
    /// 16-bit
    Word,
    /// 32-bit
    Longword,
}

#[derive(Debug)]
pub struct CompressionError {
    message: &'static str,
}

impl CompressionError {
    fn new(message: &'static str) -> CompressionError {
        return CompressionError {
            message: message,
        }
    }
}

impl Error for CompressionError {
    fn description(&self) -> &str {
        return self.message;
    }
}

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

extern {
    fn cmpr_8bit(data_stream: *const u8,
                 length: c_int,
                 data_out: *mut *mut u8,
                 compressed_size: *mut intptr_t) -> c_int;
    fn cmpr_16bit(data_stream: *const u8,
                  length: c_int,
                  data_out: *mut *mut u8,
                  compressed_size: *mut intptr_t) -> c_int;
    fn cmpr_32bit(data_stream: *const u8,
                  length: c_int,
                  data_out: *mut *mut u8,
                  compressed_size: *mut intptr_t) -> c_int;
}

/// Given a slice containing `u8`s, this function compresses the data in increments of `size`.
/// On success, returns a slice containing the compressed data.
///
/// When compressing in increments of word or longword, this function will return an error
/// if the provided data isn't an even increment of that data type.
/// Because this wraps a set of C functions, errors will be returned if the underlying
/// functions return an error; information about why the error occurred may be available
/// via stderr.
pub fn compress(data: &[u8], size: Size) -> Result<Vec<u8>, CompressionError> {
    let mut out = ptr::null_mut();
    let mut out_size : isize = 0;

    let result;

    match size {
        Size::Byte => {
            unsafe {
                result = cmpr_8bit(data.as_ptr(), data.len() as c_int, &mut out as *mut _, &mut out_size);
            };
        },
        Size::Word => {
            if data.len() % 2 != 0 {
                return Err(CompressionError::new("Provided buffer is not an even multiple of 16 bits"));
            }
            unsafe {
                result = cmpr_16bit(data.as_ptr(), data.len() as c_int / 2, &mut out as *mut _, &mut out_size);
            };
        },
        Size::Longword => {
            if data.len() % 4 != 0 {
                return Err(CompressionError::new("Provided buffer is not an even multiple of 32 bits"));
            }
            unsafe {
                result = cmpr_32bit(data.as_ptr(), data.len() as c_int / 4, &mut out as *mut _, &mut out_size);
            };
        }
    }

    let out_data;
    unsafe {
        out_data = slice::from_raw_parts(out, out_size as usize)
    };

    assert_eq!(out_size as c_int, out_data.len() as c_int); 

    if result != 0 {
        return Err(CompressionError::new("Unable to compress data!"));
    }

    let out_vec = Vec::from(out_data);
    return Ok(out_vec);
}

/// Writes a CMP header; this header is expected to come at the beginning of a compressed CMP stream.
///
/// `decompressed_size` is the size of the slice passed to `compress`,
/// while `compression_type` is the same value passed to `compress`.
pub fn create_header(decompressed_size: i32, compression_type: Size) -> Vec<u8> {
    let size_byte = match compression_type {
        Size::Byte     => 0x0,
        Size::Word     => 0x4,
        Size::Longword => 0xC,
    };

    // First word is always the size indicator
    let mut header : Vec<u8> = vec![0, size_byte];

    // 32-bit header if size is larger than 65535 bytes
    if decompressed_size > 65535 {
        // One word of padding
        header.push(0);
        header.push(0);
        // Size as 32-bit big endian
        header.push(((decompressed_size >> 24) & 0xFF) as u8);
        header.push(((decompressed_size >> 16) & 0xFF) as u8);
        header.push(((decompressed_size >> 8) & 0xFF) as u8);
        header.push((decompressed_size & 0xFF) as u8);
    // 16-bit header otherwise
    } else {
        // Size as 16-bit big endian
        header.push(((decompressed_size as i16 >> 8) & 0xFF) as u8);
        header.push((decompressed_size as i16 & 0xFF) as u8);
    }

    return header;
}
