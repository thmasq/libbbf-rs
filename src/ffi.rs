#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::CStr;
use std::fs::File;
use std::os::raw::c_char;
use std::panic;
use std::ptr;
use std::slice;

use crate::builder::BBFBuilder;
use crate::format::BBFMediaType;
use crate::reader::BBFReader;

pub struct CBbfBuilder(BBFBuilder<File>);
pub struct CBbfReader(BBFReader<File>);

#[unsafe(no_mangle)]
pub extern "C" fn bbf_builder_new(path: *const c_char) -> *mut CBbfBuilder {
    let result = panic::catch_unwind(|| {
        if path.is_null() {
            return ptr::null_mut();
        }

        let c_str = unsafe { CStr::from_ptr(path) };
        let Ok(str_slice) = c_str.to_str() else {
            return ptr::null_mut();
        };

        let Ok(file) = File::create(str_slice) else {
            return ptr::null_mut();
        };

        BBFBuilder::new(file).map_or(ptr::null_mut(), |builder| {
            Box::into_raw(Box::new(CBbfBuilder(builder)))
        })
    });

    result.unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn bbf_builder_add_page(
    builder: *mut CBbfBuilder,
    data: *const u8,
    len: usize,
    media_type: u8,
    flags: u32,
) -> u32 {
    let result = panic::catch_unwind(|| {
        if builder.is_null() || data.is_null() {
            return 0xFFFF_FFFF;
        }

        let builder_ref = unsafe { &mut (*builder).0 };
        let slice = unsafe { slice::from_raw_parts(data, len) };
        let mtype = BBFMediaType::from(media_type);

        builder_ref
            .add_page(slice, mtype, flags)
            .unwrap_or(0xFFFF_FFFF)
    });

    result.unwrap_or(0xFFFF_FFFF)
}

#[unsafe(no_mangle)]
pub extern "C" fn bbf_builder_finalize(builder: *mut CBbfBuilder) -> i32 {
    let result = panic::catch_unwind(|| {
        if builder.is_null() {
            return -1;
        }
        let builder_box = unsafe { Box::from_raw(builder) };
        match builder_box.0.finalize() {
            Ok(()) => 0,
            Err(_) => -1,
        }
    });

    result.unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn bbf_reader_open(path: *const c_char) -> *mut CBbfReader {
    let result = panic::catch_unwind(|| {
        if path.is_null() {
            return ptr::null_mut();
        }

        let c_str = unsafe { CStr::from_ptr(path) };
        let Ok(str_slice) = c_str.to_str() else {
            return ptr::null_mut();
        };

        let Ok(file) = File::open(str_slice) else {
            return ptr::null_mut();
        };

        BBFReader::new(file).map_or(ptr::null_mut(), |reader| {
            Box::into_raw(Box::new(CBbfReader(reader)))
        })
    });

    result.unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn bbf_reader_free(reader: *mut CBbfReader) {
    let _ = panic::catch_unwind(|| {
        if !reader.is_null() {
            unsafe {
                let _ = Box::from_raw(reader);
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn bbf_reader_get_page_count(reader: *mut CBbfReader) -> u32 {
    let result = panic::catch_unwind(|| {
        if reader.is_null() {
            return 0;
        }
        let reader_ref = unsafe { &(*reader).0 };
        reader_ref.footer.page_count.get()
    });

    result.unwrap_or(0)
}
