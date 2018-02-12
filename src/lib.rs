use std::slice;
use std::ptr;

extern crate libc;
use libc::{c_int, intptr_t};

pub enum Size {
    Byte,
    Word,
    Longword,
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

pub fn compress(data: &[u8], size: Size) -> Result<&[u8], &str> {
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
                return Err("Provided buffer is not an even multiple of 16 bits");
            }
            unsafe {
                result = cmpr_16bit(data.as_ptr(), data.len() as c_int / 2, &mut out as *mut _, &mut out_size);
            };
        },
        Size::Longword => {
            if data.len() % 4 != 0 {
                return Err("Provided buffer is not an even multiple of 32 bits");
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
        return Err("Unable to compress data!");
    }

    return Ok(out_data);
}
