#![no_main]
sp1_zkvm::entrypoint!(main);

use num::{BigUint, One};
use rand::Rng;
use sp1_zkvm::syscalls::syscall_uint32_sqrmod;

/// The number of limbs in a "uint32".
const N: usize = 1;

fn uint32_sqr(x: &[u8; 4], modulus: &[u8; 4]) -> [u8; 4] {
    println!("cycle-tracker-start: uint32_sqr");
    let x = x.as_ptr() as *const [u32; 8] as *const u32;
    let mut result = [0u32; 1];
    let result_ptr = result.as_mut_ptr();
    unsafe {
        core::ptr::copy(x, result_ptr, N);

        let result_ptr = result_ptr as *mut [u32; N];
        let modulus_ptr = modulus.as_ptr() as *const [u32; N];
        syscall_uint32_sqrmod(result_ptr, modulus_ptr);
    }

    println!("cycle-tracker-end: uint32_sqr");
    bytemuck::cast::<[u32; 1], [u8; 4]>(result)
}


fn biguint_to_bytes_le(x: BigUint) -> [u8; 4] {
    let mut bytes = x.to_bytes_le();
    bytes.resize(4, 0);
    bytes.try_into().unwrap()
}

#[sp1_derive::cycle_tracker]
pub fn main() {
    for _ in 0..50 {
        // Test with random numbers.
        let mut rng = rand::thread_rng();
        let mut x: [u8; 4] = rng.gen();
        let modulus: [u8; 4] = rng.gen();

        // Convert byte arrays to BigUint
        let modulus_big = BigUint::from_bytes_le(&modulus);
        let x_big = BigUint::from_bytes_le(&x);
        x = biguint_to_bytes_le(&x_big % &modulus_big);

        let result_bytes = uint32_sqr(&x, &modulus);

        let result = (x_big.pow(2)) % modulus_big;
        let result_syscall = BigUint::from_bytes_le(&result_bytes);

        assert_eq!(result, result_syscall);
    }

    // Modulus zero tests
    let modulus = [0u8; 4];
    let modulus_big: BigUint = BigUint::one() << 32;
    for _ in 0..50 {
        // Test with random numbers.
        let mut rng = rand::thread_rng();
        let mut x: [u8; 4] = rng.gen();


        // Convert byte arrays to BigUint
        let x_big = BigUint::from_bytes_le(&x);
        x = biguint_to_bytes_le(&x_big % &modulus_big);

        let result_bytes = uint32_sqr(&x, &modulus);

        let result = (x_big.pow(2)) % &modulus_big;
        let result_syscall = BigUint::from_bytes_le(&result_bytes);

        assert_eq!(result, result_syscall, "Square of {:?} with modulus 0", x);
    }

    // Hardcoded edge case: square of 1
    let modulus = [0u8; 4];

    let mut one: [u8; 4] = [0; 4];
    one[0] = 1; // Least significant byte set to 1, represents the number 1
    let result_one = uint32_sqr(&one, &modulus);
    assert_eq!(result_one, one, "Square of 1u32 should be 1u32.");

    // Hardcoded edge case: square of 0
    let zero: [u8; 4] = [0; 4]; // Represents the number 0
    let result_zero = uint32_sqr(&zero, &modulus);
    assert_eq!(result_zero, zero, "Square of 0u32 should be 0u32");

    println!("All tests passed successfully!");
}
