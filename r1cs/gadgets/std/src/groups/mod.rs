use crate::prelude::*;
use algebra::{Field, Group};
use r1cs_core::{ConstraintSystem, SynthesisError};

use std::{borrow::Borrow, fmt::Debug};

pub mod curves;

pub use self::curves::short_weierstrass::{bls12, bn, mnt};

pub trait GroupGadget<G: Group, ConstraintF: Field>:
    Sized
    + ToBytesGadget<ConstraintF>
    + EqGadget<ConstraintF>
    + ToBitsGadget<ConstraintF>
    + CondSelectGadget<ConstraintF>
    + AllocGadget<G, ConstraintF>
    + ConstantGadget<G, ConstraintF>
    + Clone
    + Debug
{
    type Value: Debug;
    type Variable;

    fn get_value(&self) -> Option<Self::Value>;

    fn get_variable(&self) -> Self::Variable;

    fn zero<CS: ConstraintSystem<ConstraintF>>(cs: CS) -> Result<Self, SynthesisError>;

    fn is_zero<CS: ConstraintSystem<ConstraintF>>(&self, cs: CS) -> Result<Boolean, SynthesisError>;

    fn add<CS: ConstraintSystem<ConstraintF>>(
        &self,
        cs: CS,
        other: &Self,
    ) -> Result<Self, SynthesisError>;

    fn sub<CS: ConstraintSystem<ConstraintF>>(
        &self,
        mut cs: CS,
        other: &Self,
    ) -> Result<Self, SynthesisError> {
        let neg_other = other.negate(cs.ns(|| "Negate other"))?;
        self.add(cs.ns(|| "Self - other"), &neg_other)
    }

    fn add_constant<CS: ConstraintSystem<ConstraintF>>(
        &self,
        cs: CS,
        other: &G,
    ) -> Result<Self, SynthesisError>;

    fn sub_constant<CS: ConstraintSystem<ConstraintF>>(
        &self,
        mut cs: CS,
        other: &G,
    ) -> Result<Self, SynthesisError> {
        let neg_other = -(*other);
        self.add_constant(cs.ns(|| "Self - other"), &neg_other)
    }

    fn double_in_place<CS: ConstraintSystem<ConstraintF>>(
        &mut self,
        cs: CS,
    ) -> Result<(), SynthesisError>;

    fn negate<CS: ConstraintSystem<ConstraintF>>(&self, cs: CS) -> Result<Self, SynthesisError>;

    /// Variable base exponentiation.
    /// Inputs must be specified in *little-endian* form.
    /// If the addition law is incomplete for the identity element,
    /// `result` must not be the identity element.
    fn mul_bits<'a, CS: ConstraintSystem<ConstraintF>>(
        &self,
        mut cs: CS,
        result: &Self,
        bits: impl Iterator<Item = &'a Boolean>,
    ) -> Result<Self, SynthesisError> {
        let mut power = self.clone();
        let mut result = result.clone();
        for (i, bit) in bits.enumerate() {
            let new_encoded = result.add(&mut cs.ns(|| format!("Add {}-th power", i)), &power)?;
            result = Self::conditionally_select(
                &mut cs.ns(|| format!("Select {}", i)),
                bit.borrow(),
                &new_encoded,
                &result,
            )?;
            power.double_in_place(&mut cs.ns(|| format!("{}-th Doubling", i)))?;
        }
        Ok(result)
    }

    fn precomputed_base_scalar_mul<'a, CS, I, B>(
        &mut self,
        mut cs: CS,
        scalar_bits_with_base_powers: I,
    ) -> Result<(), SynthesisError>
    where
        CS: ConstraintSystem<ConstraintF>,
        I: Iterator<Item = (B, &'a G)>,
        B: Borrow<Boolean>,
        G: 'a,
    {
        for (i, (bit, base_power)) in scalar_bits_with_base_powers.enumerate() {
            let new_encoded = self.add_constant(
                &mut cs.ns(|| format!("Add {}-th base power", i)),
                &base_power,
            )?;
            *self = Self::conditionally_select(
                &mut cs.ns(|| format!("Conditional Select {}", i)),
                bit.borrow(),
                &new_encoded,
                &self,
            )?;
        }
        Ok(())
    }

    /// Fixed base exponentiation, slighlty different interface from
    /// `precomputed_base_scalar_mul`. Inputs must be specified in
    /// *little-endian* form. If the addition law is incomplete for
    /// the identity element, `result` must not be the identity element.
    fn mul_bits_fixed_base<'a, CS: ConstraintSystem<ConstraintF>>(
        base: &'a G,
        mut cs: CS,
        result: &Self,
        bits: &[Boolean],
    ) -> Result<Self, SynthesisError> {
        let base_g = Self::from_value(cs.ns(|| "hardcode base"), base);
        base_g.mul_bits(cs, result, bits.into_iter())
    }

    fn precomputed_base_3_bit_signed_digit_scalar_mul<'a, CS, I, J, B>(
        _: CS,
        _: &[B],
        _: &[J],
    ) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<ConstraintF>,
        I: Borrow<[Boolean]>,
        J: Borrow<[I]>,
        B: Borrow<[G]>,
    {
        Err(SynthesisError::AssignmentMissing)
    }

    fn precomputed_base_multiscalar_mul<'a, CS, T, I, B>(
        mut cs: CS,
        bases: &[B],
        scalars: I,
    ) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<ConstraintF>,
        T: 'a + ToBitsGadget<ConstraintF> + ?Sized,
        I: Iterator<Item = &'a T>,
        B: Borrow<[G]>,
    {
        let mut result = Self::zero(&mut cs.ns(|| "Declare Result"))?;
        // Compute ∏(h_i^{m_i}) for all i.
        for (i, (bits, base_powers)) in scalars.zip(bases).enumerate() {
            let base_powers = base_powers.borrow();
            let bits = bits.to_bits(&mut cs.ns(|| format!("Convert Scalar {} to bits", i)))?;
            result.precomputed_base_scalar_mul(
                cs.ns(|| format!("Chunk {}", i)),
                bits.iter().zip(base_powers),
            )?;
        }
        Ok(result)
    }

    fn cost_of_add() -> usize;

    fn cost_of_double() -> usize;
}

#[cfg(test)]
pub(crate) mod test {
    use algebra::{Field, UniformRand};
    use r1cs_core::ConstraintSystem;

    use crate::{prelude::*, test_constraint_system::TestConstraintSystem};
    use algebra::groups::Group;
    use rand::thread_rng;

    #[allow(dead_code)]
    pub(crate) fn group_test<
        ConstraintF: Field,
        G: Group,
        GG: GroupGadget<G, ConstraintF, Value = G>,
    >()
    {
        let mut cs = TestConstraintSystem::<ConstraintF>::new();

        let a: G = UniformRand::rand(&mut thread_rng());
        let b: G = UniformRand::rand(&mut thread_rng());

        let a = GG::alloc(&mut cs.ns(|| "generate_a"), || Ok(a)).unwrap();
        let b = GG::alloc(&mut cs.ns(|| "generate_b"), || Ok(b)).unwrap();

        let zero = GG::zero(cs.ns(|| "Zero")).unwrap();
        assert_eq!(zero, zero);

        // a == a
        assert_eq!(a, a);

        // a + 0 = a
        assert_eq!(a.add(cs.ns(|| "a_plus_zero"), &zero).unwrap(), a);
        // a - 0 = a
        assert_eq!(a.sub(cs.ns(|| "a_minus_zero"), &zero).unwrap(), a);
        // a - a = 0
        assert_eq!(a.sub(cs.ns(|| "a_minus_a"), &a).unwrap(), zero);

        // a + b = b + a
        let a_b = a.add(cs.ns(|| "a_plus_b"), &b).unwrap();
        let b_a = b.add(cs.ns(|| "b_plus_a"), &a).unwrap();
        assert_eq!(a_b, b_a);
        // (a + b) + a = a + (b + a)
        let ab_a = a_b.add(&mut cs.ns(|| "a_b_plus_a"), &a).unwrap();
        let a_ba = a.add(&mut cs.ns(|| "a_plus_b_a"), &b_a).unwrap();
        assert_eq!(ab_a, a_ba);

        // a.double() = a + a
        let a_a = a.add(cs.ns(|| "a + a"), &a).unwrap();
        let mut a2 = a.clone();
        a2.double_in_place(cs.ns(|| "2a")).unwrap();
        assert_eq!(a2, a_a);
        // b.double() = b + b
        let mut b2 = b.clone();
        b2.double_in_place(cs.ns(|| "2b")).unwrap();
        let b_b = b.add(cs.ns(|| "b + b"), &b).unwrap();
        assert_eq!(b2, b_b);

        let _ = a.to_bytes(&mut cs.ns(|| "ToBytes")).unwrap();
        let _ = a.to_bytes_strict(&mut cs.ns(|| "ToBytes Strict")).unwrap();

        let _ = b.to_bytes(&mut cs.ns(|| "b ToBytes")).unwrap();
        let _ = b
            .to_bytes_strict(&mut cs.ns(|| "b ToBytes Strict"))
            .unwrap();

        mul_bits_test(GG::zero(cs.ns(|| "alloc_zero")).unwrap())
    }

    #[allow(dead_code)]
    pub(crate) fn group_test_with_unsafe_add<
        ConstraintF: Field,
        G: Group,
        GG: GroupGadget<G, ConstraintF, Value = G>,
    >()
    {
        let mut cs = TestConstraintSystem::<ConstraintF>::new();

        let a: G = UniformRand::rand(&mut thread_rng());
        let b: G = UniformRand::rand(&mut thread_rng());

        let a = GG::alloc(&mut cs.ns(|| "generate_a"), || Ok(a)).unwrap();
        let b = GG::alloc(&mut cs.ns(|| "generate_b"), || Ok(b)).unwrap();

        let zero = GG::zero(cs.ns(|| "Zero")).unwrap();
        assert_eq!(zero, zero);

        // a == a
        assert_eq!(a, a);

        // a + b = b + a
        let a_b = a.add(cs.ns(|| "a_plus_b"), &b).unwrap();
        let b_a = b.add(cs.ns(|| "b_plus_a"), &a).unwrap();
        assert_eq!(a_b, b_a);

        // (a + b) + a = a + (b + a)
        let ab_a = a_b.add(&mut cs.ns(|| "a_b_plus_a"), &a).unwrap();
        let a_ba = a.add(&mut cs.ns(|| "a_plus_b_a"), &b_a).unwrap();
        assert_eq!(ab_a, a_ba);

        // a.double() + b = (a + b) + a: Testing double() using b as shift
        let mut a2 = a.clone();
        a2.double_in_place(cs.ns(|| "2a")).unwrap();
        let a2_b = a2.add(cs.ns(|| "2a + b"), &b).unwrap();

        let a_b_a = a.add(cs.ns(|| "a + b"), &b).unwrap()
            .add(cs.ns(|| "a + b + a"), &a).unwrap();
        assert_eq!(a2_b, a_b_a);

        // (b.double() + a) = (b + a) + b: Testing double() using a as shift
        let mut b2 = b.clone();
        b2.double_in_place(cs.ns(|| "2b")).unwrap();
        let b2_a = b2.add(cs.ns(|| "2b + a"), &a).unwrap();

        let b_a_b = b.add(cs.ns(|| "b + a"), &a).unwrap()
            .add(cs.ns(|| "b + a + b"), &b).unwrap();
        assert_eq!(b2_a, b_a_b);

        let _ = a.to_bytes(&mut cs.ns(|| "ToBytes")).unwrap();
        let _ = a.to_bytes_strict(&mut cs.ns(|| "ToBytes Strict")).unwrap();

        let _ = b.to_bytes(&mut cs.ns(|| "b ToBytes")).unwrap();
        let _ = b
            .to_bytes_strict(&mut cs.ns(|| "b ToBytes Strict"))
            .unwrap();

        let shift: G = UniformRand::rand(&mut thread_rng());
        mul_bits_test(GG::alloc(cs.ns(|| "alloc random shift"), || Ok(shift)).unwrap());
    }

    #[allow(dead_code)]
    pub(crate) fn mul_bits_test<
        ConstraintF: Field,
        G: Group,
        GG: GroupGadget<G, ConstraintF, Value = G>,
    >(result: GG)
    {
        use crate::algebra::ToBits;

        for _ in 0..10 {
            let mut cs = TestConstraintSystem::<ConstraintF>::new();
            let rng = &mut thread_rng();

            let g: G = UniformRand::rand(rng);
            let gg = GG::alloc(cs.ns(|| "generate_g"), || Ok(g)).unwrap();

            let a = G::ScalarField::rand(rng);
            let b = G::ScalarField::rand(rng);
            //let ab = a * &b;
            let a_plus_b = a + &b;

            let mut a_bits = Vec::<Boolean>::alloc(cs.ns(|| "a bits"), || Ok(a.write_bits())).unwrap();
            a_bits.reverse();

            let mut b_bits = Vec::<Boolean>::alloc(cs.ns(|| "b bits"), || Ok(b.write_bits())).unwrap();
            b_bits.reverse();

            //let ab_bits = Vec::<Boolean>::alloc(cs.ns(|| "ab bits"), ||Ok(ab.write_bits())).unwrap();
            let mut a_plus_b_bits = Vec::<Boolean>::alloc(cs.ns(|| "a_plus_b bits"), || Ok(a_plus_b.write_bits())).unwrap();
            a_plus_b_bits.reverse();

            // Additivity test: a * G + b * G = (a + b) * G
            let a_times_gg_vb = {
                gg
                    .mul_bits(cs.ns(|| "a * G"), &result, a_bits.iter()).unwrap()
                    .sub(cs.ns(|| "a * G - result"), &result).unwrap()
            };
            let a_times_gg_fb = {
                GG::mul_bits_fixed_base(&g, cs.ns(|| "fb a * G"), &result, a_bits.as_slice()).unwrap()
                    .sub(cs.ns(|| "fb a * G - result"), &result).unwrap()
            };
            assert_eq!(a_times_gg_vb.get_value().unwrap(), g.mul(&a)); // Check native result
            assert_eq!(a_times_gg_fb.get_value().unwrap(), g.mul(&a)); // Check native result

            let b_times_gg_vb = {
                gg
                    .mul_bits(cs.ns(|| "b * G"), &result, b_bits.iter()).unwrap()
                    .sub(cs.ns(|| "b * G - result"), &result).unwrap()
            };
            let b_times_gg_fb = {
                GG::mul_bits_fixed_base(&g, cs.ns(|| "fb b * G"), &result, b_bits.as_slice()).unwrap()
                    .sub(cs.ns(|| "fb b * G - result"), &result).unwrap()
            };
            assert_eq!(b_times_gg_vb.get_value().unwrap(), g.mul(&b)); // Check native result
            assert_eq!(b_times_gg_fb.get_value().unwrap(), g.mul(&b)); // Check native result

            let a_plus_b_times_gg_vb = {
                gg
                    .mul_bits(cs.ns(|| "(a + b) * G"), &result, a_plus_b_bits.iter()).unwrap()
                    .sub(cs.ns(|| "(a + b) * G - result"), &result).unwrap()
            };
            let a_plus_b_times_gg_fb = {
                GG::mul_bits_fixed_base(&g, cs.ns(|| "fb (a + b) * G"), &result, a_plus_b_bits.as_slice()).unwrap()
                    .sub(cs.ns(|| "fb (a + b) * G - result"), &result).unwrap()
            };
            assert_eq!(a_plus_b_times_gg_vb.get_value().unwrap(), g.mul(&(a + &b))); // Check native result
            assert_eq!(a_plus_b_times_gg_fb.get_value().unwrap(), g.mul(&(a + &b))); // Check native result

            a_times_gg_vb
                .add(cs.ns(|| "a * G + b * G"), &b_times_gg_vb).unwrap()
                .enforce_equal(cs.ns(|| "a * G + b * G = (a + b) * G"), &a_plus_b_times_gg_vb).unwrap();

            a_times_gg_fb
                .add(cs.ns(|| "fb a * G + b * G"), &b_times_gg_fb).unwrap()
                .enforce_equal(cs.ns(|| "fb a * G + b * G = (a + b) * G"), &a_plus_b_times_gg_fb).unwrap();
        }
    }
}