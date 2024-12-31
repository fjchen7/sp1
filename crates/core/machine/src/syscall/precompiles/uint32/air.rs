use generic_array::GenericArray;
use num::{BigUint, One, Zero};
use p3_air::{Air, BaseAir};
use p3_field::{AbstractField, PrimeField32};
use p3_matrix::dense::RowMajorMatrix;
use typenum::{Unsigned, U1};
use sp1_core_executor::{ExecutionRecord, Program};
use sp1_core_executor::events::{ByteRecord, FieldOperation, PrecompileEvent};
use sp1_core_executor::syscalls::SyscallCode;
use sp1_derive::AlignedBorrow;
use sp1_stark::air::{BaseAirBuilder, InteractionScope, MachineAir, Polynomial, SP1AirBuilder};
use sp1_stark::MachineRecord;
use crate::{
    memory::{MemoryReadCols, MemoryWriteCols},
    air::MemoryAirBuilder,
    operations::{field::{
        range::FieldLtCols,
        field_op::FieldOpCols,
    }, IsZeroOperation},
    utils::{
        limbs_from_access, limbs_from_prev_access, pad_rows_fixed, words_to_bytes_le,
        words_to_bytes_le_vec,
    },
};
use std::{
    borrow::{Borrow, BorrowMut},
    mem::size_of,
};
use p3_matrix::Matrix;
use sp1_curves::{
    params::{Limbs, NumLimbs},
    uint32::U32Field,
};
use crate::memory::value_as_limbs;

#[derive(Default)]
pub struct Uint32SqrChip;

impl Uint32SqrChip {
    pub const fn new() -> Self {
        Self
    }
}

/// The number of columns in the Uint32SqrCols.
const NUM_COLS: usize = size_of::<Uint32SqrCols<u8>>();

type WordsFieldElement = U1;
const WORDS_FIELD_ELEMENT: usize = WordsFieldElement::USIZE;

/// A set of columns for the Uint32Sqr operation.
#[derive(Debug, Clone, AlignedBorrow)]
#[repr(C)]
pub struct Uint32SqrCols<T> {
    /// The shard number of the syscall.
    pub shard: T,

    /// The clock cycle of the syscall.
    pub clk: T,

    /// The pointer to the first input, the x value
    pub x_ptr: T,

    /// The pointer to the second input, the modulus value.
    pub modulus_ptr: T,

    // Memory columns.
    // x_memory is written to with the result, which is why it is of type MemoryWriteCols.
    pub x_memory: GenericArray<MemoryWriteCols<T>, WordsFieldElement>,
    pub modulus_memory: GenericArray<MemoryReadCols<T>, WordsFieldElement>,

    /// Columns for checking if modulus is zero. If it's zero, then use 2^32 as the effective
    /// modulus.
    pub modulus_is_zero: IsZeroOperation<T>,

    /// Column that is equal to is_real * (1 - modulus_is_zero.result).
    pub modulus_is_not_zero: T,

    // Output values. We compute (x * x) % modulus.
    pub output: FieldOpCols<T, U32Field>,

    pub output_range_check: FieldLtCols<T, U32Field>,

    pub is_real: T,
}


impl<F: PrimeField32> MachineAir<F> for Uint32SqrChip {
    type Record = ExecutionRecord;
    type Program = Program;

    fn name(&self) -> String {
        "Uint32SqrMod".to_string()
    }

    fn generate_trace(&self, input: &Self::Record, output: &mut Self::Record) -> RowMajorMatrix<F> {
        // Generate the trace rows & corresponding records for each chunk of events concurrently.
        let rows_and_records = input.get_precompile_events(SyscallCode::UINT32_SQR)
            .chunks(1)
            .map(|events| {
                let mut records = ExecutionRecord::default();
                let mut new_byte_lookup_events = Vec::new();

                let rows = events
                    .iter()
                    .map(|(_, event)| {
                        let event = if let PrecompileEvent::Uint32Sqr(event) = event {
                            event
                        } else {
                            unreachable!()
                        };
                        let mut row: [F; NUM_COLS] = [F::zero(); NUM_COLS];
                        let cols: &mut Uint32SqrCols<F> = row.as_mut_slice().borrow_mut();

                        // Decode uint32 points
                        let x = BigUint::from_bytes_le(&words_to_bytes_le::<4>(&event.x));
                        let modulus =
                            BigUint::from_bytes_le(&words_to_bytes_le::<4>(&event.modulus));

                        // Assign basic values to the columns.
                        cols.is_real = F::one();
                        cols.shard = F::from_canonical_u32(event.shard);
                        cols.clk = F::from_canonical_u32(event.clk);
                        cols.x_ptr = F::from_canonical_u32(event.x_ptr);
                        cols.modulus_ptr = F::from_canonical_u32(event.modulus_ptr);

                        // Populate memory columns.
                        for i in 0..WORDS_FIELD_ELEMENT {
                            cols.x_memory[i]
                                .populate(event.x_memory_records[i], &mut new_byte_lookup_events);
                            cols.modulus_memory[i].populate(
                                event.modulus_memory_records[i],
                                &mut new_byte_lookup_events,
                            );
                        }

                        let modulus_bytes = words_to_bytes_le_vec(&event.modulus);
                        let modulus_byte_sum = modulus_bytes.iter().map(|b| *b as u32).sum::<u32>();
                        IsZeroOperation::populate(&mut cols.modulus_is_zero, modulus_byte_sum);

                        // Populate the output column.
                        let effective_modulus =
                            if modulus.is_zero() { BigUint::one() << 32 } else { modulus.clone() };
                        let result = cols.output.populate_with_modulus(
                            &mut new_byte_lookup_events,
                            event.shard,
                            &x,
                            &x,
                            &effective_modulus,
                            // &modulus,
                            FieldOperation::Mul,
                        );

                        cols.modulus_is_not_zero = F::one() - cols.modulus_is_zero.result;
                        if cols.modulus_is_not_zero == F::one() {
                            cols.output_range_check.populate(
                                &mut new_byte_lookup_events,
                                event.shard,
                                &result,
                                &effective_modulus,
                            );
                        }

                        row
                    })
                    .collect::<Vec<_>>();
                records.add_byte_lookup_events(new_byte_lookup_events);
                (rows, records)
            })
            .collect::<Vec<_>>();


        //  Generate the trace rows for each event.
        let mut rows = Vec::new();
        for (row, mut record) in rows_and_records {
            rows.extend(row);
            output.append(&mut record);
        }


        pad_rows_fixed(
            &mut rows,
            || {
                let mut row: [F; NUM_COLS] = [F::zero(); NUM_COLS];
                let cols: &mut Uint32SqrCols<F> = row.as_mut_slice().borrow_mut();

                let x = BigUint::zero();
                let y = BigUint::zero();
                cols.output.populate(&mut vec![], 0, &x, &y, FieldOperation::Mul);

                row
            },
            input.fixed_log2_rows::<F, _>(self),
        );

        // Convert the trace to a row major matrix.
        RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_COLS)
    }

    fn included(&self, shard: &Self::Record) -> bool {
        if let Some(shape) = shard.shape.as_ref() {
            shape.included::<F, _>(self)
        } else {
            !shard.get_precompile_events(SyscallCode::UINT32_SQR).is_empty()
        }
    }

    fn local_only(&self) -> bool {
        true
    }
}


impl<F> BaseAir<F> for Uint32SqrChip {
    fn width(&self) -> usize {
        NUM_COLS
    }
}


impl<AB> Air<AB> for Uint32SqrChip
where
    AB: SP1AirBuilder,
    Limbs<AB::Var, <U32Field as NumLimbs>::Limbs>: Copy,

{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let local: &Uint32SqrCols<AB::Var> = (*local).borrow();

        // We are computing (x * x) % modulus. The value of x is stored in the "prev_value" of
        // the x_memory, since we write to it later.
        let x_limbs = limbs_from_prev_access(&local.x_memory);
        let modulus_limbs = limbs_from_access(&local.modulus_memory);

        // If the modulus is zero, then we don't perform the modulus operation.
        // Evaluate the modulus_is_zero operation by summing each byte of the modulus. The sum will
        // not overflow because we are summing 4 bytes.
        let modulus_byte_sum =
            modulus_limbs.0.iter().fold(AB::Expr::zero(), |acc, &limb| acc + limb);
        IsZeroOperation::<AB::F>::eval(
            builder,
            modulus_byte_sum,
            local.modulus_is_zero,
            local.is_real.into(),
        );

        // If the modulus is zero, we'll actually use 2^32 as the modulus, so nothing happens.
        // Otherwise, we use the modulus passed in.
        let modulus_is_zero = local.modulus_is_zero.result;
        let mut coeff_2_32 = Vec::new();
        coeff_2_32.resize(4, AB::Expr::zero());
        coeff_2_32.push(AB::Expr::one());
        let modulus_polynomial: Polynomial<AB::Expr> = modulus_limbs.into();
        let p_modulus: Polynomial<AB::Expr> = modulus_polynomial
            * (AB::Expr::one() - modulus_is_zero.into())
            + Polynomial::from_coefficients(&coeff_2_32) * modulus_is_zero.into();

        // Evaluate the uint32 multiplication
        local.output.eval_with_modulus(
            builder,
            &x_limbs,
            &x_limbs,
            &p_modulus,
            FieldOperation::Mul,
            local.is_real,
        );

        // Verify the range of the output if the moduls is not zero.  Also, check the value of
        // modulus_is_not_zero.
        local.output_range_check.eval(
            builder,
            &local.output.result,
            &modulus_limbs,
            local.modulus_is_not_zero,
        );
        builder.assert_eq(
            local.modulus_is_not_zero,
            local.is_real * (AB::Expr::one() - modulus_is_zero.into()),
        );

        // Assert that the correct result is being written to x_memory.
        builder
            .when(local.is_real)
            .assert_all_eq(local.output.result, value_as_limbs(&local.x_memory));

        // Read and write x.
        builder.eval_memory_access_slice(
            local.shard,
            local.clk.into() + AB::Expr::one(),
            local.x_ptr,
            &local.x_memory,
            local.is_real,
        );

        // Evaluate the modulus_ptr memory access.
        builder.eval_memory_access_slice(
            local.shard,
            local.clk.into(),
            local.modulus_ptr,
            &local.modulus_memory,
            local.is_real,
        );

        // Receive the arguments.
        builder.receive_syscall(
            local.shard,
            local.clk,
            AB::F::from_canonical_u32(SyscallCode::UINT32_SQR.syscall_id()),
            local.x_ptr,
            local.modulus_ptr,
            local.is_real,
            InteractionScope::Local,
        );

        // Assert that is_real is a boolean.
        builder.assert_bool(local.is_real);
    }
}