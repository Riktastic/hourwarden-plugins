//! SDK for HourWarden WebAssembly plugins (plugin API 2). It implements the
//! host ABI from `docs/plugins/protocol-wasm.md` so a plugin only writes its
//! logic: it reads an event and calls [`emit`], [`log`] and [`capability`].
//!
//! A plugin crate (`crate-type = ["cdylib"]`) ends with the [`plugin!`] macro,
//! which exports the entry points the app calls:
//!
//! ```ignore
//! use hourwarden_plugin as hw;
//! use serde_json::{json, Value};
//!
//! fn on_event(ev: Value) {
//!     if ev["kind"] == "foreground" { /* ... */ hw::emit(&json!({ ... })); }
//! }
//! hw::plugin!(event = on_event);
//! ```

// INPUT is a single-threaded wasm scratch buffer; the 2024 lint does not apply.
#![allow(static_mut_refs)]

pub use serde_json::{json, Value};

/// The host functions, the only way a plugin reaches anything outside itself.
/// On wasm they are imported from the app; off wasm they are no-ops, so a
/// plugin's own logic can be unit-tested with `cargo test` on the host.
#[cfg(target_arch = "wasm32")]
mod ffi {
    #[link(wasm_import_module = "hw")]
    extern "C" {
        pub fn emit(ptr: i32, len: i32);
        pub fn log(ptr: i32, len: i32);
        pub fn capability(name_ptr: i32, name_len: i32, arg_ptr: i32, arg_len: i32, out_ptr: i32, out_cap: i32) -> i32;
    }
}
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::missing_safety_doc)]
mod ffi {
    pub unsafe fn emit(_ptr: i32, _len: i32) {}
    pub unsafe fn log(_ptr: i32, _len: i32) {}
    pub unsafe fn capability(_np: i32, _nl: i32, _ap: i32, _al: i32, _op: i32, _oc: i32) -> i32 {
        0
    }
}

/// Reused buffer the host writes an event's JSON into; see [`alloc`].
static mut INPUT: Vec<u8> = Vec::new();

/// Allocates (reusing one buffer) the memory the host writes an event into.
/// Called by the app, not by plugin code; exported through [`plugin!`].
#[doc(hidden)]
pub fn alloc(len: i32) -> i32 {
    let len = len.max(0) as usize;
    unsafe {
        INPUT = vec![0u8; len];
        INPUT.as_mut_ptr() as i32
    }
}

/// Parses the JSON the host wrote at `(ptr, len)`; used by [`plugin!`].
#[doc(hidden)]
pub fn read_input(ptr: i32, len: i32) -> Value {
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len.max(0) as usize) };
    serde_json::from_slice(bytes).unwrap_or(Value::Null)
}

/// Reports one message to the app: `detail`, `meeting`, `calendar`, `network`,
/// `file_changed`, `metric` or `status` (the API 1 shapes).
pub fn emit(message: &Value) {
    let bytes = serde_json::to_vec(message).unwrap_or_default();
    unsafe { ffi::emit(bytes.as_ptr() as i32, bytes.len() as i32) };
}

/// Sends `state` ("running", "degraded" or "failing") and a message to the
/// plugin's status in the app.
pub fn status(state: &str, message: &str) {
    emit(&json!({ "method": "status", "params": { "state": state, "message": message } }));
}

/// Writes a line to the plugin's log in the app.
pub fn log(message: &str) {
    unsafe { ffi::log(message.as_ptr() as i32, message.len() as i32) };
}

/// Asks the app for a capability, returning its JSON result. `None` means the
/// capability is unknown or had nothing to return. The buffer grows and the
/// call repeats when the first result was larger than the buffer.
pub fn capability(name: &str, arg: &Value) -> Option<Value> {
    let arg_bytes = if arg.is_null() { Vec::new() } else { serde_json::to_vec(arg).ok()? };
    let mut out = vec![0u8; 1024];
    loop {
        let n = unsafe {
            ffi::capability(
                name.as_ptr() as i32,
                name.len() as i32,
                arg_bytes.as_ptr() as i32,
                arg_bytes.len() as i32,
                out.as_mut_ptr() as i32,
                out.len() as i32,
            )
        };
        if n <= 0 {
            return None;
        }
        let n = n as usize;
        if n > out.len() {
            out = vec![0u8; n];
            continue;
        }
        return serde_json::from_slice(&out[..n]).ok();
    }
}

/// Exports the ABI entry points the app calls, wiring them to the plugin's
/// handlers. `event` is required; `init` (called once, with settings and
/// customer folders) is optional.
#[macro_export]
macro_rules! plugin {
    (event = $event:path $(, init = $init:path)? $(,)?) => {
        #[no_mangle]
        pub extern "C" fn hw_abi_version() -> i32 {
            2
        }
        #[no_mangle]
        pub extern "C" fn hw_alloc(len: i32) -> i32 {
            $crate::alloc(len)
        }
        #[no_mangle]
        pub extern "C" fn hw_event(ptr: i32, len: i32) {
            let event = $crate::read_input(ptr, len);
            $event(event);
        }
        $(
            #[no_mangle]
            pub extern "C" fn hw_init(ptr: i32, len: i32) {
                let data = $crate::read_input(ptr, len);
                $init(data);
            }
        )?
    };
}
