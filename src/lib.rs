use std::slice;
use std::ptr;

extern crate libc;
use libc::{c_int, intptr_t};

extern {
    fn cmpr_8bit(data_stream: *const u8,
                 length: c_int,
                 data_out: *mut *mut u8,
                 compressed_size: *mut intptr_t) -> c_int;
}

pub fn compress_8bit(data: &[u8]) -> Result<&[u8], &str> {
    let mut out = ptr::null_mut();
    let mut out_size : isize = 0;

    let result;
    unsafe {
        result = cmpr_8bit(data.as_ptr(), data.len() as c_int, &mut out as *mut _, &mut out_size);
    };

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
