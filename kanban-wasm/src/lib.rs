//! kanban's layout logic, callable from a browser.
//!
//! This exists so the published page can show what a message will look like
//! before you have a kanban running locally. Running it still needs the local
//! server - a browser cannot name a process - but the preview does not.
//!
//! The interface is deliberately small: strings cross the boundary as UTF-8
//! buffers, several of them joined by newlines. Messages reject control
//! characters, so a newline is an unambiguous separator, and this avoids
//! shipping a second JSON parser into the wasm module.
//!
//! Callers must return every pointer they receive to `kanban_free`.

use std::alloc::{alloc, dealloc, Layout};

/// Reserve a buffer for the caller to write UTF-8 into.
///
/// # Safety
/// The returned pointer is valid for `len` bytes and must be released with
/// [`kanban_free`] using the same length.
#[no_mangle]
pub unsafe extern "C" fn kanban_alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::null_mut();
    }
    match Layout::from_size_align(len, 1) {
        Ok(layout) => alloc(layout),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Release a buffer obtained from [`kanban_alloc`] or [`kanban_layout`].
///
/// # Safety
/// `ptr` must have come from this module and `len` must be the length it was
/// created with.
#[no_mangle]
pub unsafe extern "C" fn kanban_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    if let Ok(layout) = Layout::from_size_align(len, 1) {
        dealloc(ptr, layout);
    }
}

/// Lay out a message for a mode, and return the rows joined by newlines.
///
/// `mode` follows the order the CLI lists them in: 0 single, 1 multiple,
/// 2 multiple2, 3 long, 4 vertical, 5 wave. Anything else is treated as single,
/// matching the server.
///
/// # Safety
/// `ptr`/`len` must describe a valid UTF-8 buffer, and `out_len` must point at
/// a writable `usize`. The returned pointer must be released with
/// [`kanban_free`] using the length written to `out_len`.
#[no_mangle]
pub unsafe extern "C" fn kanban_layout(
    mode: u32,
    ptr: *const u8,
    len: usize,
    thread: usize,
    length: usize,
    out_len: *mut usize,
) -> *mut u8 {
    let input = if ptr.is_null() || len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).into_owned()
    };

    // The plural modes take one message per line; the rest take the lot as a
    // single message, spaces included.
    let words: Vec<String> = input.split('\n').map(|s| s.to_string()).collect();
    let first = words.first().cloned().unwrap_or_default();

    let lines = match mode {
        1 => kanban_core::multiple(&first, thread),
        2 => kanban_core::multiple2(&words),
        3 => kanban_core::long(&first, length),
        4 => kanban_core::vertical(&words),
        5 => kanban_core::wave(&first, length),
        _ => kanban_core::single(&first),
    };

    let joined = lines.join("\n").into_bytes();
    let size = joined.len();
    if !out_len.is_null() {
        *out_len = size;
    }
    if size == 0 {
        return std::ptr::null_mut();
    }

    let Ok(layout) = Layout::from_size_align(size, 1) else {
        return std::ptr::null_mut();
    };
    let out = alloc(layout);
    if !out.is_null() {
        std::ptr::copy_nonoverlapping(joined.as_ptr(), out, size);
    }
    out
}
