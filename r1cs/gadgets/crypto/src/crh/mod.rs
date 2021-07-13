use algebra::{
    Field, PrimeField
};
use std::fmt::Debug;

use primitives::crh::{
    FieldBasedHash, FixedLengthCRH
};
use r1cs_core::{ConstraintSystem, SynthesisError};

use r1cs_std::prelude::*;

pub mod bowe_hopwood;
pub mod injective_map;
pub mod pedersen;

pub mod sbox;
pub use self::sbox::*;

pub mod poseidon;
pub use self::poseidon::*;
use primitives::{AlgebraicSponge, SpongeMode};
use r1cs_std::fields::fp::FpGadget;

pub trait FixedLengthCRHGadget<H: FixedLengthCRH, ConstraintF: Field>: Sized {
    type OutputGadget: EqGadget<ConstraintF>
    + ToBytesGadget<ConstraintF>
    + CondSelectGadget<ConstraintF>
    + AllocGadget<H::Output, ConstraintF>
    + Debug
    + Clone
    + Sized;
    type ParametersGadget: AllocGadget<H::Parameters, ConstraintF> + Clone;

    fn check_evaluation_gadget<CS: ConstraintSystem<ConstraintF>>(
        cs: CS,
        parameters: &Self::ParametersGadget,
        input: &[UInt8],
    ) -> Result<Self::OutputGadget, SynthesisError>;
}

pub trait FieldBasedHashGadget<H: FieldBasedHash<Data = ConstraintF>, ConstraintF: Field>: Sized {
    type DataGadget: FieldGadget<ConstraintF, ConstraintF>;

    fn enforce_hash_constant_length<CS: ConstraintSystem<ConstraintF>>(
        cs: CS,
        input: &[Self::DataGadget],
    ) -> Result<Self::DataGadget, SynthesisError>;
}

pub trait FieldHasherGadget<
    H: FieldBasedHash<Data = ConstraintF>,
    ConstraintF: Field,
    HG: FieldBasedHashGadget<H, ConstraintF>
>
{
    fn enforce_hash<CS: ConstraintSystem<ConstraintF>>(
        &self,
        cs: CS,
        personalization: Option<&[HG::DataGadget]>
    ) -> Result<HG::DataGadget, SynthesisError>;
}

pub trait AlgebraicSpongeGadget<H: AlgebraicSponge<ConstraintF>, ConstraintF: PrimeField>:
    ConstantGadget<H, ConstraintF>
    + From<Vec<FpGadget<ConstraintF>>>
    + Sized
{
    type DataGadget: FieldGadget<ConstraintF, ConstraintF>;

    fn new<CS: ConstraintSystem<ConstraintF>>(cs: CS) -> Result<Self, SynthesisError>;

    fn get_state(&self) -> &[FpGadget<ConstraintF>];

    fn set_state(&mut self, state: Vec<FpGadget<ConstraintF>>);

    fn get_mode(&self) -> &SpongeMode;

    fn set_mode(&mut self, mode: SpongeMode);

    fn enforce_absorb<CS: ConstraintSystem<ConstraintF>>(
        &mut self,
        cs: CS,
        input: &[Self::DataGadget],
    ) -> Result<(), SynthesisError>;

    fn enforce_squeeze<CS: ConstraintSystem<ConstraintF>>(
        &mut self,
        cs: CS,
        num: usize,
    ) -> Result<Vec<Self::DataGadget>, SynthesisError>;
}

#[cfg(test)]
mod test {
    use algebra::PrimeField;
    use primitives::{FieldBasedHash, AlgebraicSponge};
    use crate::{FieldBasedHashGadget, AlgebraicSpongeGadget};
    use r1cs_std::{
        fields::fp::FpGadget,
        test_constraint_system::TestConstraintSystem,
        alloc::AllocGadget,
    };
    use r1cs_core::ConstraintSystem;

    pub(crate) fn constant_length_field_based_hash_gadget_native_test<
        F: PrimeField,
        H: FieldBasedHash<Data = F>,
        HG: FieldBasedHashGadget<H, F, DataGadget = FpGadget<F>>
    >(inputs: Vec<F>)
    {
        let mut cs = TestConstraintSystem::<F>::new();

        let primitive_result = {
            let mut digest = H::init_constant_length(inputs.len(), None);
            inputs.iter().for_each(|elem| { digest.update(*elem); });
            digest.finalize().unwrap()
        };

        let mut input_gadgets = Vec::with_capacity(inputs.len());
        inputs.into_iter().enumerate().for_each(|(i, elem)| {
            let elem_gadget = HG::DataGadget::alloc(
                cs.ns(|| format!("alloc input {}", i)),
                || Ok(elem)
            ).unwrap();
            input_gadgets.push(elem_gadget);
        });

        let gadget_result = HG::enforce_hash_constant_length(
            cs.ns(|| "check_poseidon_gadget"),
            input_gadgets.as_slice()
        ).unwrap();

        assert_eq!(primitive_result, gadget_result.value.unwrap());

        if !cs.is_satisfied(){
            println!("{:?}", cs.which_is_unsatisfied());
        }
        assert!(cs.is_satisfied());
    }

    pub(crate) fn algebraic_sponge_gadget_native_test<
        F: PrimeField,
        H: AlgebraicSponge<F>,
        HG: AlgebraicSpongeGadget<H, F, DataGadget = FpGadget<F>>
    >(inputs: Vec<F>)
    {
        use std::collections::HashSet;

        let mut cs = TestConstraintSystem::<F>::new();

        // Check equality between primitive and gadget result
        let mut primitive_sponge = H::init();
        primitive_sponge.absorb(inputs.clone());

        let mut input_gadgets = Vec::with_capacity(inputs.len());
        inputs.iter().enumerate().for_each(|(i, elem)|{
            let elem_gadget = HG::DataGadget::alloc(
                cs.ns(|| format!("alloc input {}", i)),
                || Ok(elem.clone())
            ).unwrap();
            input_gadgets.push(elem_gadget);
        });

        let mut sponge_gadget = HG::new(cs.ns(|| "new poseidon sponge")).unwrap();
        sponge_gadget.enforce_absorb(cs.ns(|| "absorb inputs"), input_gadgets.as_slice()).unwrap();

        for i in 0..inputs.len() {
            let output_gadgets = sponge_gadget.enforce_squeeze(
                cs.ns(|| format!("squeeze {} field elements",  i + 1)),
                i + 1
            ).unwrap().iter().map(|fe_gadget| fe_gadget.value.unwrap()).collect::<Vec<_>>();
            assert_eq!(output_gadgets, primitive_sponge.squeeze(i + 1));
        }

        // Check squeeze() outputs the correct number of field elements
        // all different from each others
        let mut set = HashSet::new();
        for i in 0..=10 {

            let outs = sponge_gadget.enforce_squeeze(
                cs.ns(|| format!("test squeeze {} field elements",  i)),
                i
            ).unwrap();
            assert_eq!(i, outs.len());

            // HashSet::insert(val) returns false if val was already present, so to check
            // that all the elements output by the sponge are different, we assert insert()
            // returning always true
            outs.into_iter().for_each(|f| assert!(set.insert(f.value.unwrap())));
        }

        assert!(cs.is_satisfied());
    }
}