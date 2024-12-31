use typenum::{U4, U7};

use num::{BigUint, One};
use serde::{Deserialize, Serialize};

use crate::params::{FieldParameters, NumLimbs};

/// Although `U32` is technically not a field, we utilize `FieldParameters` here for compatibility.
/// This approach is specifically for the `FieldOps` multiplication operation, which employs these
/// parameters solely as a modulus, rather than enforcing the requirement of being a proper field.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct U32Field;

impl FieldParameters for U32Field {
    /// The modulus of the field. It is represented as a little-endian array of 5 bytes.
    const MODULUS: &'static [u8] = &[
        0, 0, 0, 0, 1,
    ];

    /// A rough witness-offset estimate given the size of the limbs and the size of the field.
    const WITNESS_OFFSET: usize = 1usize << 14;

    /// The modulus of Uint32 is 2^32.
    fn modulus() -> BigUint {
        BigUint::one() << 32
    }
}

impl NumLimbs for U32Field {
    type Limbs = U4;
    // Note we use one more limb than usual because for mulmod with mod 1<<32, we need an extra
    // limb.
    type Witness = U7;
}
