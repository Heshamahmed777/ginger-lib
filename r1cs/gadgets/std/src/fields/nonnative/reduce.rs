//! A submodule of low-level function for modular reduction/normalization of non-native field gadgets.
use algebra::{biginteger::BigInteger, fields::{PrimeField, FpParameters}};

use crate::{
    prelude::*,
    fields::{
        fp::FpGadget,
        nonnative::{
            params::get_params, nonnative_field_gadget::NonNativeFieldGadget
        }
    },
    overhead
};
use r1cs_core::{ConstraintSystem, SynthesisError};
use std::{cmp::min, marker::PhantomData, vec::Vec};

use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::{Zero, One};

use crate::fields::FieldGadget;

/// Recombines the large integer value from a vector of (not necessarily normalized) limbs. 
pub fn limbs_to_bigint<ConstraintF: PrimeField>(
    bits_per_limb: usize,
    limbs: &[ConstraintF],
) -> BigUint {
    let mut val = BigUint::zero();
    let mut big_cur = BigUint::one();
    let two = BigUint::from(2u32);
    for limb in limbs.iter().rev() {
        let mut limb_repr = limb.into_repr().to_bits();
        limb_repr.reverse(); //We need them in little endian
        let mut small_cur = big_cur.clone();
        for limb_bit in limb_repr.iter() {
            if *limb_bit {
                val += &small_cur;
            }
            small_cur *= 2u32;
        }
        big_cur *= two.pow(bits_per_limb as u32);
    }

    val
}

/// Converts an unsigned big integer `bigint` into an element from the constraint field F_p by
/// computing (bigint mod p).
pub fn bigint_to_constraint_field<ConstraintF: PrimeField>(bigint: &BigUint) -> ConstraintF {
    let mut val = ConstraintF::zero();
    let mut cur = ConstraintF::one();
    let bytes = bigint.to_bytes_be();

    let basefield_256 = ConstraintF::from_repr(<ConstraintF as PrimeField>::BigInt::from(256));

    for byte in bytes.iter().rev() {
        let bytes_basefield = ConstraintF::from(*byte as u128);
        val += cur * bytes_basefield;

        cur *= &basefield_256;
    }

    val
}

/// The collections of methods for reducing the presentations of NonNativeFieldGadgets
pub struct Reducer<SimulationF: PrimeField, ConstraintF: PrimeField> {
    pub simulation_phantom: PhantomData<SimulationF>,
    pub constraint_phantom: PhantomData<ConstraintF>,
}

impl<SimulationF: PrimeField, ConstraintF: PrimeField> Reducer<SimulationF, ConstraintF> {
    /// Convert a limb of `num_bits` into a vector of Boolean gadgets.
    /// Allowing `num_bits` to be at most `ConstraintF::size_in_bits() - 1` for efficient 'unpacking'.
    // Consumes `num_bits` + 1 constraints.
    pub fn limb_to_bits<CS: ConstraintSystem<ConstraintF>>(
        mut cs: CS,
        limb: &FpGadget<ConstraintF>,
        num_bits: usize,
    ) -> Result<Vec<Boolean>, SynthesisError> {

        let num_bits = min(num_bits, ConstraintF::Params::CAPACITY as usize);

        // we extract the bits from the `num_bits` least significant bits
        let to_skip = ConstraintF::size_in_bits() - num_bits;
        assert!(to_skip > 0
            && to_skip <= ConstraintF::Params::CAPACITY as usize
        );

        limb.to_bits_with_length_restriction(cs.ns(|| "limb to bits"), to_skip)
    }

    /// Reduction to normal form, which again has no excess in its limbs.
    pub fn reduce<CS: ConstraintSystem<ConstraintF>>(
        mut cs: CS,
        elem: &mut NonNativeFieldGadget<SimulationF, ConstraintF>
    ) -> Result<(), SynthesisError> {
        // Alloc of a non-native in normal form (not necessarily below the 
        // non-native modulus)
        let new_elem = NonNativeFieldGadget::alloc(
            cs.ns(|| "alloc normal form"),
            || { Ok(elem.get_value().unwrap_or_default()) }
        )?;
        elem.enforce_equal(cs.ns(|| "elem == new_elem"),&new_elem)?;
        *elem = new_elem;
        Ok(())
    }

    /// Reduction to be enforced after additions. 
    /// Checks if the resulting elem is still "small" enough for a multiplication, and 
    /// reduces it otherwise.
    pub fn post_add_reduce<CS: ConstraintSystem<ConstraintF>>(
        cs: CS,
        elem: &mut NonNativeFieldGadget<SimulationF, ConstraintF>
    ) -> Result<(), SynthesisError> {
        let params = get_params(SimulationF::size_in_bits(), ConstraintF::size_in_bits());
        let surfeit = overhead!(elem.num_of_additions_over_normal_form + ConstraintF::one()) + 1;

        if ConstraintF::size_in_bits() > 2 * params.bits_per_limb + surfeit + 1 {
            Ok(())
        } else {
            Self::reduce(cs, elem)
        }
    }

    /// Reduction used before multiplication to reduce the representations in a way that allows efficient multiplication
    pub fn pre_mul_reduce<CS: ConstraintSystem<ConstraintF>>(
        mut cs: CS,
        elem: &mut NonNativeFieldGadget<SimulationF, ConstraintF>,
        elem_other: &mut NonNativeFieldGadget<SimulationF, ConstraintF>
    ) -> Result<(), SynthesisError> {
        let params = get_params(SimulationF::size_in_bits(), ConstraintF::size_in_bits());

        if 2 * params.bits_per_limb + algebra::log2(params.num_limbs) as usize
            > ConstraintF::size_in_bits() - 1
        {
            panic!("The current limb parameters do not support multiplication.");
        }

        // TODO: Try to understand if one can write the optional reduction of one
        // (or both) of the factors more elegantly.  
        let mut i = 0;
        loop {
            let prod_of_num_of_additions = (elem.num_of_additions_over_normal_form
                + ConstraintF::one())
                * (elem_other.num_of_additions_over_normal_form + ConstraintF::one());
            // overhead_limb = (num_add(a) + 1)(num_add(b)+1) * num_limbs
            // Why the product of the num_of_additions and not the sum?
            let overhead_limb = overhead!(prod_of_num_of_additions.mul(
                &ConstraintF::from_repr(<ConstraintF as PrimeField>::BigInt::from(
                    (params.num_limbs) as u64
                ))
            ));
            let bits_per_mulresult_limb = 2 * (params.bits_per_limb + 1) + overhead_limb;

            // if the product of limbs has bit length < than length(modulus),
            // there is nothing to do.
            if bits_per_mulresult_limb < ConstraintF::size_in_bits() {
                break;
            }

            // otherwise we reduce the factor which is expected to have larger excess
            // over normal form.
            if elem.num_of_additions_over_normal_form
                >= elem_other.num_of_additions_over_normal_form
            {
                Self::reduce(cs.ns(|| format!("reduce elem {}", i)), elem)?;
            } else {
                Self::reduce(cs.ns(|| format!("reduce elem other {}", i)),elem_other)?;
            }
            i += 1;
        }

        Ok(())
    }

    /// Reduction to the normal form
    pub fn pre_eq_reduce<CS: ConstraintSystem<ConstraintF>>(
        cs: CS,
        elem: &mut NonNativeFieldGadget<SimulationF, ConstraintF>,
    ) -> Result<(), SynthesisError> {
        if elem.is_in_the_normal_form {
            return Ok(());
        }

        Self::reduce(cs, elem)
    }

    /// Group and check equality, the low-level function for equality checks?
    pub fn group_and_check_equality<CS: ConstraintSystem<ConstraintF>>(
        mut cs: CS,
        surfeit: usize,
        bits_per_limb: usize,
        shift_per_limb: usize,
        left: &[FpGadget<ConstraintF>],
        right: &[FpGadget<ConstraintF>],
    ) -> Result<(), SynthesisError> {

        let zero = FpGadget::<ConstraintF>::zero(cs.ns(|| "hardcode zero"))?;

        let mut limb_pairs = Vec::<(FpGadget<ConstraintF>, FpGadget<ConstraintF>)>::new();
        let num_limb_in_a_group = (ConstraintF::size_in_bits()
            - 1
            - surfeit
            - 1
            - 1
            - 1
            - (bits_per_limb - shift_per_limb))
            / shift_per_limb;

        let shift_array = {
            let mut array = Vec::new();
            let mut cur = ConstraintF::one().into_repr();
            for _ in 0..num_limb_in_a_group {
                array.push(ConstraintF::from_repr(cur));
                cur.muln(shift_per_limb as u32);
            }

            array
        };

        for (left_limb, right_limb) in left.iter().zip(right.iter()).rev() {
            // note: the `rev` operation is here, so that the first limb (and the first groupped limb) will be the least significant limb.
            limb_pairs.push((left_limb.clone(), right_limb.clone()));
        }

        let mut groupped_limb_pairs = Vec::<(FpGadget<ConstraintF>, FpGadget<ConstraintF>, usize)>::new();

        for (i, limb_pairs_in_a_group) in limb_pairs.chunks(num_limb_in_a_group).enumerate() {
            let mut left_total_limb = zero.clone();
            let mut right_total_limb = zero.clone();

            for (j, ((left_limb, right_limb), shift)) in
                limb_pairs_in_a_group.iter().zip(shift_array.iter()).enumerate()
                {
                    let left_mul = left_limb.mul_by_constant(
                        cs.ns(|| format!("left_limb * shift {},{}", i, j)),
                        shift
                    )?;
                    left_total_limb.add_in_place(
                        cs.ns(|| format!("left_total_limb += left_mul {},{}", i, j)),
                        &left_mul
                    )?;

                    let right_mul = right_limb.mul_by_constant(
                        cs.ns(|| format!("right_limb * shift {},{}", i, j)),
                        shift
                    )?;
                    right_total_limb.add_in_place(
                        cs.ns(|| format!("right_total_limb += right_mul {},{}", i, j)),
                        &right_mul
                    )?;
                }

            groupped_limb_pairs.push((
                left_total_limb,
                right_total_limb,
                limb_pairs_in_a_group.len(),
            ));
        }

        // This part we mostly use the techniques in bellman-bignat
        // The following code is adapted from https://github.com/alex-ozdemir/bellman-bignat/blob/master/src/mp/bignat.rs#L567
        let mut carry_in = zero;
        let mut carry_in_value = ConstraintF::zero();
        let mut accumulated_extra = BigUint::zero();
        for (group_id, (left_total_limb, right_total_limb, num_limb_in_this_group)) in
            groupped_limb_pairs.iter().enumerate()
            {
                let mut pad_limb_repr: <ConstraintF as PrimeField>::BigInt = ConstraintF::one().into_repr();

                pad_limb_repr.muln(
                    (surfeit
                        + (bits_per_limb - shift_per_limb)
                        + shift_per_limb * num_limb_in_this_group
                        + 1
                        + 1) as u32,
                );
                let pad_limb = ConstraintF::from_repr(pad_limb_repr);

                let left_total_limb_value = left_total_limb.get_value().unwrap_or_default();
                let right_total_limb_value = right_total_limb.get_value().unwrap_or_default();

                let mut carry_value =
                    left_total_limb_value + carry_in_value + pad_limb - right_total_limb_value;

                let mut carry_repr = carry_value.into_repr();
                carry_repr.divn((shift_per_limb * num_limb_in_this_group) as u32);

                carry_value = ConstraintF::from_repr(carry_repr);

                let carry = FpGadget::<ConstraintF>::alloc(
                    cs.ns(|| format!("alloc carry {}", group_id)),
                    || Ok(carry_value)
                )?;

                accumulated_extra += limbs_to_bigint(bits_per_limb, &[pad_limb]);

                let (new_accumulated_extra, remainder) = accumulated_extra.div_rem(
                    &BigUint::from(2u64).pow((shift_per_limb * num_limb_in_this_group) as u32),
                );
                let remainder_limb = bigint_to_constraint_field::<ConstraintF>(&remainder);

                // Now check
                //      left_total_limb + pad_limb + carry_in - right_total_limb
                //   =  carry shift by (shift_per_limb * num_limb_in_this_group) + remainder

                let eqn_left = left_total_limb
                    .add_constant(
                        cs.ns(|| format!("left_total_limb + pad_limb {}", group_id)),
                        &pad_limb
                    )?
                    .add(
                        cs.ns(|| format!("left_total_limb + pad_limb + carry_in {}", group_id)),
                        &carry_in
                    )?
                    .sub(
                        cs.ns(|| format!("left_total_limb + pad_limb + carry_in - right_total_limb {}", group_id)),
                        right_total_limb
                    )?;

                let eqn_right = carry
                    .mul_by_constant(
                        cs.ns(|| format!("carry * 2^(shift_per_limb * num_limb_in_this_group) {}", group_id)),
                        &ConstraintF::from(2u64).pow(&[(shift_per_limb * num_limb_in_this_group) as u64])
                    )?
                    .add_constant(
                        cs.ns(|| format!("carry * 2^(shift_per_limb * num_limb_in_this_group) + remainder_limb {}", group_id)),
                        &remainder_limb
                    )?;

                eqn_left.enforce_equal(
                    cs.ns(|| format!("eqn_left == eqn_right {}", group_id)),
                    &eqn_right
                )?;

                accumulated_extra = new_accumulated_extra;
                carry_in = carry.clone();
                carry_in_value = carry_value;

                if group_id == groupped_limb_pairs.len() - 1 {
                    let accumulated_extra_g = FpGadget::<ConstraintF>::from_value(
                        cs.ns(|| format!("hardcode accumulated_extra {}", group_id)),
                        &bigint_to_constraint_field(&accumulated_extra)
                    );
                    carry.enforce_equal(
                        cs.ns(|| format!("carry == accumulated_extra {}", group_id)),
                        &accumulated_extra_g
                    )?;
                } else {
                    Reducer::<SimulationF, ConstraintF>::limb_to_bits(
                        cs.ns(|| format!("carry_to_bits_{}", group_id)),
                        &carry,
                        surfeit + bits_per_limb
                    )?;
                }
            }

        Ok(())
    }
}
