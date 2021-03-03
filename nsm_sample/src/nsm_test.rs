use nsm::{nsm_get_attestation_doc, nsm_get_random, nsm_lib_init};
use std::ptr::null;

pub fn test_nsm() {
    println!("begin of test nsm");

    let mut attestation_buf = [0u8; 1024 * 8];
    let mut random_buf = [0u8; 256];
    let mut attestation_len = attestation_buf.len() as u32;
    let mut random_len = random_buf.len();

    unsafe {
        let fd = nsm_lib_init();
        println!("fd: {}", fd);
        let mut err_code = nsm_get_attestation_doc(
            fd,
            null(),
            0,
            null(),
            0,
            null(),
            0,
            attestation_buf.as_mut_ptr(),
            &mut attestation_len,
        );
        println!("get attestation error code: {:?}", err_code);

        err_code = nsm_get_random(fd, random_buf.as_mut_ptr(), &mut random_len);
        println!("get random error code: {:?}", err_code);
    }

    println!("actual attestation length: {}", attestation_len);
    println!("attestation result: {:?}", attestation_buf);

    println!("actual random length: {}", random_len);
    println!("random result: {:?}", random_buf);

    println!("end of test nsm");
}
