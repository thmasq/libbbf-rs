use std::ffi::CStr;
use std::fs::File;
use std::os::raw::c_char;
use std::slice;

use crate::builder::BBFBuilder;
use crate::format::BBFMediaType;
use crate::reader::BBFReader;

pub struct CBbfBuilder(BBFBuilder<File>);
pub struct CBbfReader(BBFReader<File>);

#[unsafe(no_mangle)]
pub extern "C" fn bbf_builder_new(path: *const c_char) -> *mut CBbfBuilder {
    let c_str = unsafe { CStr::from_ptr(path) };
    let str_slice = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let file = match File::create(str_slice) {
        Ok(f) => f,
        Err(_) => return std::ptr::null_mut(),
    };

    match BBFBuilder::new(file) {
        Ok(builder) => Box::into_raw(Box::new(CBbfBuilder(builder))),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn bbf_builder_add_page(
    builder: *mut CBbfBuilder,
    data: *const u8,
    len: usize,
    media_type: u8,
) -> u32 {
    let builder = unsafe { &mut (*builder).0 };
    let slice = unsafe { slice::from_raw_parts(data, len) };
    let mtype = BBFMediaType::from(media_type);

    match builder.add_page(slice, mtype) {
        Ok(idx) => idx,
        Err(_) => 0xFFFFFFFF,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn bbf_builder_finalize(builder: *mut CBbfBuilder) -> i32 {
    if builder.is_null() {
        return -1;
    }
    let builder_box = unsafe { Box::from_raw(builder) };
    match builder_box.0.finalize() {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn bbf_reader_open(path: *const c_char) -> *mut CBbfReader {
    let c_str = unsafe { CStr::from_ptr(path) };
    let str_slice = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let file = match File::open(str_slice) {
        Ok(f) => f,
        Err(_) => return std::ptr::null_mut(),
    };

    match BBFReader::new(file) {
        Ok(reader) => Box::into_raw(Box::new(CBbfReader(reader))),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn bbf_reader_free(reader: *mut CBbfReader) {
    if !reader.is_null() {
        unsafe {
            let _ = Box::from_raw(reader);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn bbf_reader_get_page_count(reader: *mut CBbfReader) -> u32 {
    let reader = unsafe { &(*reader).0 };
    reader.footer.page_count.get()
}
