use algebra::PrimeField;
use r1cs_core::{ConstraintSystemAbstract, SynthesisError};
use r1cs_std::{to_field_gadget_vec::ToConstraintFieldGadget, fields::fp::FpGadget, ToBytesGadget, boolean::Boolean};

/// the trait for Fiat-Shamir RNG Gadget
pub trait FiatShamirRngGadget<ConstraintF: PrimeField>:
    Sized
    + Clone
{
    /// initialize an empty transcript
    fn init<CS: ConstraintSystemAbstract<ConstraintF>>(cs: CS) -> Result<Self, SynthesisError>;

    /// initialize from a seed
    fn init_from_seed<CS: ConstraintSystemAbstract<ConstraintF>>(
        cs: CS,
        seed: Vec<ConstraintF>
    ) -> Result<Self, SynthesisError>;

    /// take in field elements
    fn enforce_absorb<CS, T>(
        &mut self,
        cs: CS,
        elems: &[T]
    ) -> Result<(), SynthesisError>
    where
        CS: ConstraintSystemAbstract<ConstraintF>,
        T: ToConstraintFieldGadget<ConstraintF, FieldGadget = FpGadget<ConstraintF>>
           + ToBytesGadget<ConstraintF>;

    /// Output field elements
    fn enforce_squeeze<CS: ConstraintSystemAbstract<ConstraintF>>(
        &mut self,
        cs: CS,
        num: usize
    ) -> Result<Vec<FpGadget<ConstraintF>>, SynthesisError>;

    /// Output non-native field elements of 128 bits
    fn enforce_squeeze_128_bits_challenges<CS: ConstraintSystemAbstract<ConstraintF>>(
        &mut self,
        cs: CS,
        num: usize
    ) -> Result<Vec<[Boolean; 128]>, SynthesisError>;
}