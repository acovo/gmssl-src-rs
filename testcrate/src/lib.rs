extern crate libc;

use std::ffi::{c_schar, CString};
use libc::{c_char, c_ulong};

extern "C" {
    pub fn gmssl_version_num() -> c_ulong;
    pub fn gmssl_version_str() -> *mut c_char;
}

#[test]
fn version_code_works() {
    unsafe {
        println!("{}", gmssl_version_num());
        assert!(gmssl_version_num() > 0);
    }
}

#[test]
fn version_name_works() {
    unsafe {
        let ret = gmssl_version_str();
        let ret_str = std::ffi::CStr::from_ptr(ret);
        println!("{:?}", ret_str);
        assert!(ret_str==std::ffi::CString::new("GmSSL 3.1.1 Dev").unwrap_or(std::ffi::CString::default()).as_c_str());
    }
}
