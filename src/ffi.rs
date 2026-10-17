use crate::decider::SeedDecider;
use crate::types::{CandidateAction, GeometryMetrics};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn seed_decider_create(
    profile: *const c_char,
    target: *const c_char,
) -> *mut SeedDecider {
    let p_str = if profile.is_null() {
        "HOST_SOVEREIGN"
    } else {
        unsafe { CStr::from_ptr(profile).to_str().unwrap_or("HOST_SOVEREIGN") }
    };
    let t_str = if target.is_null() {
        "host"
    } else {
        unsafe { CStr::from_ptr(target).to_str().unwrap_or("host") }
    };
    let decider = Box::new(SeedDecider::new(p_str, t_str));
    Box::into_raw(decider)
}

#[no_mangle]
pub extern "C" fn seed_decider_evaluate(
    decider_ptr: *mut SeedDecider,
    candidate_json: *const c_char,
    geom_json: *const c_char,
) -> *mut c_char {
    if decider_ptr.is_null() || candidate_json.is_null() {
        return std::ptr::null_mut();
    }
    let decider = unsafe { &*decider_ptr };
    let c_str = unsafe { CStr::from_ptr(candidate_json).to_str().unwrap_or("") };
    let g_str = if geom_json.is_null() {
        ""
    } else {
        unsafe { CStr::from_ptr(geom_json).to_str().unwrap_or("") }
    };

    let candidate: CandidateAction = match serde_json::from_str(c_str) {
        Ok(c) => c,
        Err(_) => return std::ptr::null_mut(),
    };
    let geom: GeometryMetrics = if g_str.is_empty() {
        GeometryMetrics::default()
    } else {
        serde_json::from_str(g_str).unwrap_or_default()
    };

    let verdict = decider.evaluate(&candidate, &geom);
    let out_json = serde_json::to_string(&verdict).unwrap_or_default();
    CString::new(out_json).map(|c| c.into_raw()).unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn seed_decider_free(ptr: *mut SeedDecider) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr)) };
    }
}

#[no_mangle]
pub extern "C" fn seed_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { drop(CString::from_raw(ptr)) };
    }
}
