extern crate libc;

use libc::c_ulong;

extern "C" {
    pub fn gmssl_version_num() -> c_ulong;
}

#[test]
fn version_works() {
    unsafe {
        println!("{:#x}", gmssl_version_num());
        assert!(gmssl_version_num() > 0);
    }
}
