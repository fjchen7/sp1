use p3_air::{Air, BaseAir};
use p3_field::PrimeField32;
use p3_matrix::dense::RowMajorMatrix;
use typenum::{Unsigned, U1};
use sp1_core_executor::{ExecutionRecord, Program};
use sp1_derive::AlignedBorrow;
use sp1_stark::air::{MachineAir, SP1AirBuilder};

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
}


impl<F: PrimeField32> MachineAir<F> for Uint32SqrChip {
    type Record = ExecutionRecord;
    type Program = Program;

    fn name(&self) -> String {
        todo!()
    }

    fn generate_trace(&self, input: &Self::Record, output: &mut Self::Record) -> RowMajorMatrix<F> {
        todo!()
    }

    fn included(&self, shard: &Self::Record) -> bool {
        todo!()
    }

    fn local_only(&self) -> bool {
        true
    }
}


impl<F> BaseAir<F> for Uint32SqrChip {
    fn width(&self) -> usize {
        todo!()
    }
}


impl<AB> Air<AB> for Uint32SqrChip
where
    AB: SP1AirBuilder,
{
    fn eval(&self, builder: &mut AB) {
        todo!()
    }
}