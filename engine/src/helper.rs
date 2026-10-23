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
) -> *mut libc::c_char {
    let result = std::panic::catch_unwind(|| {
        let text = unsafe { CStr::from_ptr(value) }.to_str()?;
        dispatch(text, uid)
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
