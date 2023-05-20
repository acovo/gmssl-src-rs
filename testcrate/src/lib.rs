extern crate libc;

use std::ffi::{c_schar, c_void, CString};
use std::{ptr, slice};
use libc::{c_char, c_uchar, c_uint, c_ulong};

extern "C" {
    pub fn gmssl_version_num() -> c_ulong;
    pub fn gmssl_version_str() -> *mut c_char;
    pub fn sm3_digest(data:*const c_uchar, datalen:c_uint, dgst: *mut [u8;32]) ->c_void;
}


fn print_hex(data:Vec<u8>) {
    for pos in 0..data.len() {
        print!("{:02X} ",data.get(pos).unwrap());
        if pos%16 ==15 {
            println!("");
        }
    }
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

#[test]
fn sm3_digest_works() {
    unsafe {

        let output_max_len = 32;

        let plain_text = std::ffi::CString::new("data-test-sm3-1").unwrap_or(CString::default());

        let mut digest_data:[u8;32]=[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];

        sm3_digest(plain_text.as_ptr() as *const c_uchar, plain_text.as_bytes().len() as c_uint, &mut digest_data);

        print_hex(digest_data.to_vec());
    }
}