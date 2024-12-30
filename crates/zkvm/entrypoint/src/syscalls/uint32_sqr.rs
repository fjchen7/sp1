#[cfg(target_os = "zkvm")]
use core::arch::asm;

/// Uint32 square operation.
///
/// The result is written over the first input.
///
/// ### Safety
///
/// The caller must ensure that `x` and `modulus` are valid pointers to data that is aligned along a four
/// byte boundary.
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn syscall_uint32_sqrmod(x: *mut [u32; 1], modulus: *const [u32; 1]) {
    #[cfg(target_os = "zkvm")]
    unsafe {
        asm!(
            "ecall",
            in("t0") crate::syscalls::UINT32_SQR,
            in("a0") x,
            in("a1") modulus,
        );
    }

    #[cfg(not(target_os = "zkvm"))]
    unreachable!()
}
