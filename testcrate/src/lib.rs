extern crate libc;

use std::ffi::{c_schar, c_void, CString};
use std::{ptr, slice};
use libc::{c_char, c_uchar, c_uint, c_ulong};

const SM4_KEY_SIZE:usize=16;
const SM4_NUM_ROUNDS:usize=32;
const SM4_BLOCK_SIZE:usize=16;

#[repr(C)]
pub struct Sm4Key {
     pub rk: [u32;SM4_KEY_SIZE*2]    //256bit rk
}

extern "C" {
    pub fn gmssl_version_num() -> c_ulong;
    pub fn gmssl_version_str() -> *mut c_char;
    pub fn sm3_digest(data:*const c_uchar, datalen:c_uint, dgst: *mut [u8;32]) ->c_void;
    pub fn sm4_set_encrypt_key(key: &mut Sm4Key, user_key:&[u8;16]);
    pub fn sm4_set_decrypt_key(key: &mut Sm4Key, user_key:&[u8;16]);
    pub fn sm4_encrypt(key:&Sm4Key,in_data:&[u8;SM4_BLOCK_SIZE],out:*mut [u8;SM4_BLOCK_SIZE])->c_void;
}

fn print_hex(data:Vec<u8>) {
    for pos in 0..data.len() {
        print!("{:02X} ",data.get(pos).unwrap());
        if pos%16 ==15 {
            println!("");
        }
    }
}

fn print_hex_u32(data:Vec<u32>) {
    for pos in 0..data.len() {
        print!("{:08X} ",data.get(pos).unwrap());
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

#[test]
fn sm4_encrypt_works() {
    unsafe {

        let mut sm4_key:Sm4Key=Sm4Key{
            rk:[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
        };

        let user_key:[u8;16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
            0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
        ];

        sm4_set_encrypt_key(&mut sm4_key, &user_key);

        println!("sm4-rk ->");
        print_hex_u32(sm4_key.rk.to_vec());

        let rk_expect:[u32;32] = [
            0xf12186f9, 0x41662b61, 0x5a6ab19a, 0x7ba92077,
            0x367360f4, 0x776a0c61, 0xb6bb89b3, 0x24763151,
            0xa520307c, 0xb7584dbd, 0xc30753ed, 0x7ee55b57,
            0x6988608c, 0x30d895b7, 0x44ba14af, 0x104495a1,
            0xd120b428, 0x73b55fa3, 0xcc874966, 0x92244439,
            0xe89e641f, 0x98ca015a, 0xc7159060, 0x99e1fd2e,
            0xb79bd80c, 0x1d2115b0, 0x0e228aeb, 0xf1780c81,
            0x428d3654, 0x62293496, 0x01cf72e5, 0x9124a012,
        ];

        assert!(sm4_key.rk==rk_expect);

        let plaintext:[u8;16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
            0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
        ];

        let mut ciphertext:[u8;16] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
        sm4_encrypt(&sm4_key,&plaintext,&mut ciphertext );

        println!("cipher_text ->");
        print_hex(ciphertext.to_vec());


        let ciphertext_expect:[u8;16] = [
            0x68, 0x1e, 0xdf, 0x34, 0xd2, 0x06, 0x96, 0x5e,
            0x86, 0xb3, 0xe9, 0x4f, 0x53, 0x6e, 0x42, 0x46,
        ];

        assert!(ciphertext==ciphertext_expect);


        sm4_set_decrypt_key(&mut sm4_key, &user_key);
        println!("sm4-rk ->");
        print_hex_u32(sm4_key.rk.to_vec());

        let mut plaintext_decrypt:[u8;16] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
        sm4_encrypt(&sm4_key, &ciphertext, &mut plaintext_decrypt);

        assert!(plaintext_decrypt==plaintext);
    }
}