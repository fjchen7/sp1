mod air;

pub use air::*;

#[cfg(test)]
mod tests {

    use sp1_core_executor::Program;
    use sp1_curves::{params::FieldParameters, utils::biguint_from_limbs};
    use sp1_curves::uint32::U32Field;
    use sp1_stark::CpuProver;
    use test_artifacts::UINT32_SQR_ELF;

    use crate::{
        io::SP1Stdin,
        utils::{self, run_test},
    };

    #[test]
    fn test_uint32_sqr() {
        utils::setup_logger();
        let program = Program::from(UINT32_SQR_ELF).unwrap();
        run_test::<CpuProver<_, _>>(program, SP1Stdin::new()).unwrap();
    }

    #[test]
    fn test_uint32_modulus() {
        assert_eq!(biguint_from_limbs(U32Field::MODULUS), U32Field::modulus());
    }
}
