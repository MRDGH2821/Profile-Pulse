//! Minimal WASM profile pic source plugin for Profile Pulse Phase 3.

use core::cell::UnsafeCell;
use serde::{Deserialize, Serialize};

struct SyncCell<T>(UnsafeCell<T>);
unsafe impl<T> Sync for SyncCell<T> {}

impl<T> SyncCell<T> {
    const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    fn with<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
        f(self.0.get())
    }
}

static HEAP: SyncCell<[u8; 524288]> = SyncCell::new([0; 524288]);
static HEAP_OFF: SyncCell<usize> = SyncCell::new(0);

#[derive(Serialize)]
struct Candidate {
    source_key: String,
    label: String,
    preview_url: Option<String>,
    fetch_token: String,
}

#[derive(Deserialize)]
struct ContactContext {
    emails: Vec<String>,
}

#[derive(Serialize)]
struct PicBytes {
    content_type: String,
    bytes: Vec<u8>,
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_reset() {
    HEAP_OFF.with(|off| unsafe {
        *off = 0;
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_alloc(size: i32) -> i32 {
    if size <= 0 {
        return -1;
    }
    let size = size as usize;
    HEAP.with(|heap| {
        HEAP_OFF.with(|off| unsafe {
            let current = *off;
            if current + size > (*heap).len() {
                return -1;
            }
            let ptr = (*heap).as_ptr().add(current) as i32;
            *off = current + size;
            ptr
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn discover(
    ctx_ptr: i32,
    ctx_len: i32,
    out_ptr: i32,
    out_cap: i32,
) -> i32 {
    if ctx_ptr <= 0 || out_ptr <= 0 || ctx_len < 0 || out_cap <= 0 {
        return -1;
    }
    let ctx_bytes = unsafe { core::slice::from_raw_parts(ctx_ptr as *const u8, ctx_len as usize) };
    let ctx: ContactContext = match serde_json::from_slice(ctx_bytes) {
        Ok(v) => v,
        Err(_) => return -2,
    };
    let label = if let Some(email) = ctx.emails.first() {
        format!("Hello WASM ({email})")
    } else {
        "Hello WASM".to_string()
    };
    let candidates = vec![Candidate {
        source_key: "hello".into(),
        label,
        preview_url: None,
        fetch_token: "hello".into(),
    }];
    let json = match serde_json::to_vec(&candidates) {
        Ok(v) => v,
        Err(_) => return -3,
    };
    write_output(out_ptr, out_cap, &json)
}

#[unsafe(no_mangle)]
pub extern "C" fn fetch_pic(
    token_ptr: i32,
    token_len: i32,
    out_ptr: i32,
    out_cap: i32,
) -> i32 {
    if token_ptr <= 0 || out_ptr <= 0 || token_len < 0 || out_cap <= 0 {
        return -1;
    }
    let token_bytes =
        unsafe { core::slice::from_raw_parts(token_ptr as *const u8, token_len as usize) };
    let token: String = match serde_json::from_slice(token_bytes) {
        Ok(v) => v,
        Err(_) => return -2,
    };
    if token != "hello" {
        return -404;
    }
    let png: [u8; 67] = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
        0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
        0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
        0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
        0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let payload = PicBytes {
        content_type: "image/png".into(),
        bytes: png.to_vec(),
    };
    let json = match serde_json::to_vec(&payload) {
        Ok(v) => v,
        Err(_) => return -3,
    };
    write_output(out_ptr, out_cap, &json)
}

fn write_output(out_ptr: i32, out_cap: i32, data: &[u8]) -> i32 {
    if data.len() > out_cap as usize {
        return -4;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(data.as_ptr(), out_ptr as *mut u8, data.len());
    }
    data.len() as i32
}
