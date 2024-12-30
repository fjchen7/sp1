use num::{BigUint, One, Zero};

use sp1_primitives::consts::{bytes_to_words_le, words_to_bytes_le_vec};

use crate::{
    events::{PrecompileEvent, Uint32SqrEvent},
    syscalls::{Syscall, SyscallCode, SyscallContext},
};

const WORDS_FIELD_ELEMENT: usize = 1;
pub(crate) struct Uint32SqrSyscall;

impl Syscall for Uint32SqrSyscall {
    fn execute(
        &self,
        rt: &mut SyscallContext,
        syscall_code: SyscallCode,
        arg1: u32,
        arg2: u32,
    ) -> Option<u32> {
        let clk = rt.clk;

        let x_ptr = arg1;
        if x_ptr % 4 != 0 {
            panic!();
        }

        let modulus_ptr = arg2;
        if modulus_ptr % 4 != 0 {
            panic!();
        }

        // First read the words for the x value. We can read a slice_unsafe here because we write
        // the computed result to x later.
        let x = rt.slice_unsafe(x_ptr, WORDS_FIELD_ELEMENT);

        // Read the u32 modulus value.
        let (modulus_memory_records, modulus) = rt.mr_slice(modulus_ptr, WORDS_FIELD_ELEMENT);

        // Get the BigUint values for x and the modulus.
        let uint32_x = BigUint::from_bytes_le(&words_to_bytes_le_vec(&x));
        let uint32_modulus = BigUint::from_bytes_le(&words_to_bytes_le_vec(&modulus));

        // Perform the multiplication and take the result modulo the modulus.
        let result: BigUint = if uint32_modulus.is_zero() {
            let modulus = BigUint::one() << 32;
            uint32_x.pow(2) % modulus
        } else {
            uint32_x.pow(2) % uint32_modulus
        };

        let mut result_bytes = result.to_bytes_le();
        result_bytes.resize(4, 0u8); // Pad the result to 4 bytes.

        // Convert the result to little endian u32 words.
        let result = bytes_to_words_le::<1>(&result_bytes);

        // Increment clk so that the write is not at the same cycle as the read.
        rt.clk += 1;
        // Write the result to x and keep track of the memory records.
        let x_memory_records = rt.mw_slice(x_ptr, &result);

        let lookup_id = rt.syscall_lookup_id;
        let shard = rt.current_shard();
        let event = PrecompileEvent::Uint32Sqr(Uint32SqrEvent {
            lookup_id,
            shard,
            clk,
            x_ptr,
            x,
            modulus_ptr,
            modulus,
            x_memory_records,
            modulus_memory_records,
            local_mem_access: rt.postprocess(),
        });
        let sycall_event =
            rt.rt.syscall_event(clk, syscall_code.syscall_id(), arg1, arg2, lookup_id);
        rt.add_precompile_event(syscall_code, sycall_event, event);

        None
    }

    fn num_extra_cycles(&self) -> u32 {
        1
    }
}
