//! Optional administrator service. Narrow authenticated XPC; never executes supplied commands.
use anyhow::Result;
use serde_json::{json, Value};
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct GateRequest {
    protocol: u32,
    operation: GateOperation,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
enum GateOperation {
    Health,
}
use std::ffi::{CStr, CString};
#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn coqui_helper_listen() -> i32;
    fn coqui_helper_request(payload: *const libc::c_char) -> *mut libc::c_char;
}
pub fn request(value: &Value) -> Result<Value> {
    #[cfg(target_os = "macos")]
    unsafe {
        let value = CString::new(serde_json::to_string(value)?)?;
        let raw = coqui_helper_request(value.as_ptr());
        anyhow::ensure!(!raw.is_null(), "Helper unavailable");
        let response = CStr::from_ptr(raw).to_string_lossy().into_owned();
        libc::free(raw.cast());
        let result: Value = serde_json::from_str(&response)?;
        anyhow::ensure!(
            result.get("error").is_none(),
            "{}",
            result["error"]
                .as_str()
                .unwrap_or("Helper rejected request")
        );
        Ok(result)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = value;
        anyhow::bail!("macOS helper unsupported")
    }
}
pub fn serve() -> Result<()> {
    #[cfg(target_os = "macos")]
    unsafe {
        anyhow::ensure!(
            coqui_helper_listen() == 0,
            "Helper requires root-owned package installation"
        );
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        anyhow::bail!("macOS helper unsupported")
    }
}
fn dispatch(request: &str, uid: u32) -> Result<Value> {
    anyhow::ensure!(uid >= 501, "Interactive user required");
    let r: GateRequest = serde_json::from_str(request)?;
    anyhow::ensure!(r.protocol == 1, "Helper protocol mismatch");
    match r.operation {
        GateOperation::Health => Ok(
            json!({"protocol":1,"version":env!("CARGO_PKG_VERSION"),"uid":uid,"effectiveUid":unsafe{libc::geteuid()},"state":"connected","mode":"installationGate","qualification":"Authenticated installation gate; no scan or removal operation is exposed yet."}),
        ),
    }
}
#[no_mangle]
pub unsafe extern "C" fn coqui_helper_dispatch(
    value: *const libc::c_char,
    uid: u32,
    pid: i32,
    production: i32,
) -> *mut libc::c_char {
    let result = std::panic::catch_unwind(|| {
        let text = unsafe { CStr::from_ptr(value) }.to_str()?;
        if production != 0 {
            crate::privileged::dispatch(text, uid, pid)
        } else {
            dispatch(text, uid)
        }
    })
    .unwrap_or_else(|_| Err(anyhow::anyhow!("Helper request failed")));
    let response = result.unwrap_or_else(|e| json!({"error":e.to_string()}));
    CString::new(response.to_string()).unwrap().into_raw()
}
#[no_mangle]
pub unsafe extern "C" fn coqui_helper_free(value: *mut libc::c_char) {
    if !value.is_null() {
        drop(unsafe { CString::from_raw(value) });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg(target_os = "macos")]
    fn invalid_code_hash_is_rejected_before_xpc_and_never_unwinds_into_rust() {
        unsafe extern "C" {
            fn coqui_valid_signing_requirement(text: *const libc::c_char) -> i32;
        }
        for text in ["", "identifier broken and cdhash H\"\"", "cdhash H\"xyz\""] {
            let value = CString::new(text).unwrap();
            assert_eq!(
                unsafe { coqui_valid_signing_requirement(value.as_ptr()) },
                0
            );
        }
        let value = CString::new("identifier \"com.coqui.CoquiPCCleaner.Helper\" and cdhash H\"50bae86f32aae7f5af81668d1fc04717cb596f46\"").unwrap();
        assert_eq!(
            unsafe { coqui_valid_signing_requirement(value.as_ptr()) },
            1
        );
    }
    #[test]
    #[cfg(target_os = "macos")]
    fn forged_authorization_forms_fail_without_interaction() {
        unsafe extern "C" {
            fn coqui_authorization_valid(form: *const libc::c_char) -> i32;
        }
        for form in ["not-base64", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="] {
            let form = CString::new(form).unwrap();
            assert_eq!(unsafe { coqui_authorization_valid(form.as_ptr()) }, 0);
        }
    }
    #[test]
    fn gate_denies_unknown_operations_and_protocols() {
        assert!(dispatch(
            r#"{"protocol":1,"operation":"execute","command":"rm"}"#,
            501
        )
        .is_err());
        assert!(dispatch(r#"{"protocol":2,"operation":"health"}"#, 501).is_err());
        assert!(dispatch(r#"{"protocol":1,"operation":"health"}"#, 0).is_err());
        assert!(dispatch(
            r#"{"protocol":1,"operation":"health","command":"ignored"}"#,
            501
        )
        .is_err());
        assert_eq!(
            dispatch(r#"{"protocol":1,"operation":"health"}"#, 501).unwrap()["uid"],
            501
        );
    }
}

/// Only typed operations reach this client; authorization forms never cross the webview boundary.
pub fn production(operation: crate::privileged::Operation, authorized: bool) -> Result<Value> {
    production_with_cancel(
        operation,
        authorized,
        &std::sync::atomic::AtomicBool::new(false),
    )
}
pub fn production_with_cancel(
    operation: crate::privileged::Operation,
    authorized: bool,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<Value> {
    let mut envelope =
        serde_json::json!({"protocol":2,"version":env!("CARGO_PKG_VERSION"),"request":operation});
    if authorized {
        let challenge = production(
            crate::privileged::Operation::Challenge {
                request: Box::new(operation),
            },
            false,
        )?;
        envelope["challenge"] = challenge;
    }
    #[cfg(target_os = "macos")]
    let result = if authorized {
        unsafe {
            unsafe extern "C" {
                fn coqui_helper_authorized_request(
                    payload: *const libc::c_char,
                    cancel: *const std::ffi::c_void,
                ) -> *mut libc::c_char;
            }
            let text = CString::new(envelope.to_string())?;
            let raw = coqui_helper_authorized_request(
                text.as_ptr(),
                (cancel as *const std::sync::atomic::AtomicBool).cast(),
            );
            anyhow::ensure!(!raw.is_null(), "Authorization unavailable");
            let text = CStr::from_ptr(raw).to_string_lossy().into_owned();
            libc::free(raw.cast());
            serde_json::from_str::<Value>(&text)?
        }
    } else {
        request(&envelope)?
    };
    #[cfg(not(target_os = "macos"))]
    let result = request(&envelope)?;
    if let Some(error) = result.get("error") {
        anyhow::bail!("{}", error.as_str().unwrap_or("Helper request failed"));
    }
    anyhow::ensure!(
        result["protocol"] == 2 && result["version"] == env!("CARGO_PKG_VERSION"),
        "Helper response version mismatch"
    );
    result
        .get("result")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Incomplete helper response"))
}
pub fn protocol() -> u32 {
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn coqui_helper_protocol() -> i32;
        }
        unsafe { coqui_helper_protocol() as u32 }
    }
    #[cfg(not(target_os = "macos"))]
    {
        0
    }
}

pub fn begin_authorization(cancel: &std::sync::atomic::AtomicBool) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn coqui_authorization_begin(cancel: *const std::ffi::c_void) -> u32;
        }
        anyhow::ensure!(
            unsafe {
                coqui_authorization_begin((cancel as *const std::sync::atomic::AtomicBool).cast())
            } > 0,
            "Administrator authorization was cancelled, declined or unavailable"
        );
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = cancel;
        anyhow::bail!("Administrator authorization requires macOS")
    }
}
pub fn clear_authorization() {
    #[cfg(target_os = "macos")]
    unsafe {
        unsafe extern "C" {
            fn coqui_authorization_clear();
        }
        coqui_authorization_clear();
    }
}
pub fn authorization_remaining() -> u32 {
    #[cfg(target_os = "macos")]
    unsafe {
        unsafe extern "C" {
            fn coqui_authorization_remaining() -> u32;
        }
        coqui_authorization_remaining()
    }
    #[cfg(not(target_os = "macos"))]
    {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn coqui_is_cancelled(pointer: *const std::ffi::c_void) -> i32 {
    if pointer.is_null() {
        return 0;
    }
    unsafe { &*pointer.cast::<std::sync::atomic::AtomicBool>() }
        .load(std::sync::atomic::Ordering::SeqCst) as i32
}
