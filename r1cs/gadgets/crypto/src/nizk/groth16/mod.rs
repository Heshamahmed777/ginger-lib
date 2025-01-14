use algebra::{AffineCurve, Field, PairingEngine, ToConstraintField};
use proof_systems::groth16::{Parameters, PreparedVerifyingKey, Proof, VerifyingKey};
use r1cs_core::{ConstraintSynthesizer, ConstraintSystemAbstract, SynthesisError};
use r1cs_std::prelude::*;
use std::{borrow::Borrow, marker::PhantomData};

use super::{NIZKVerifierGadget, NIZK};

/// Note: V should serialize its contents to `Vec<E::Fr>` in the same order as
/// during the constraint generation.
pub struct Groth16<
    E: PairingEngine,
    C: ConstraintSynthesizer<E::Fr>,
    V: ToConstraintField<E::Fr> + ?Sized,
> {
    #[doc(hidden)]
    _engine: PhantomData<E>,
    #[doc(hidden)]
    _circuit: PhantomData<C>,
    #[doc(hidden)]
    _verifier_input: PhantomData<V>,
}

impl<E: PairingEngine, C: ConstraintSynthesizer<E::Fr>, V: ToConstraintField<E::Fr> + ?Sized> NIZK
    for Groth16<E, C, V>
{
    type Circuit = C;
    type AssignedCircuit = C;
    type ProvingParameters = Parameters<E>;
    type VerificationParameters = VerifyingKey<E>;
    type PreparedVerificationParameters = PreparedVerifyingKey<E>;
    type VerifierInput = V;
    type Proof = Proof<E>;
}

#[derive(Derivative)]
#[derivative(Clone(bound = "P::G1Gadget: Clone, P::G2Gadget: Clone"))]
pub struct ProofGadget<
    PairingE: PairingEngine,
    ConstraintF: Field,
    P: PairingGadget<PairingE, ConstraintF>,
> {
    pub a: P::G1Gadget,
    pub b: P::G2Gadget,
    pub c: P::G1Gadget,
}

#[derive(Derivative)]
#[derivative(Clone(
    bound = "P::G1Gadget: Clone, P::GTGadget: Clone, P::G1PreparedGadget: Clone, \
             P::G2PreparedGadget: Clone, "
))]
pub struct VerifyingKeyGadget<
    PairingE: PairingEngine,
    ConstraintF: Field,
    P: PairingGadget<PairingE, ConstraintF>,
> {
    pub alpha_g1_beta_g2: P::GTGadget,
    pub gamma_g2: P::G2Gadget,
    pub delta_g2: P::G2Gadget,
    pub gamma_abc_g1: Vec<P::G1Gadget>,
}

impl<PairingE: PairingEngine, ConstraintF: Field, P: PairingGadget<PairingE, ConstraintF>>
    VerifyingKeyGadget<PairingE, ConstraintF, P>
{
    pub fn prepare<CS: ConstraintSystemAbstract<ConstraintF>>(
        &self,
        mut cs: CS,
    ) -> Result<PreparedVerifyingKeyGadget<PairingE, ConstraintF, P>, SynthesisError> {
        let mut cs = cs.ns(|| "Preparing verifying key");

        let gamma_g2_neg = self.gamma_g2.negate(&mut cs.ns(|| "Negate gamma_g2"))?;
        let gamma_g2_neg_pc = P::prepare_g2(&mut cs.ns(|| "Prepare gamma_g2_neg"), &gamma_g2_neg)?;

        let delta_g2_neg = self.delta_g2.negate(&mut cs.ns(|| "Negate delta_g2"))?;
        let delta_g2_neg_pc = P::prepare_g2(&mut cs.ns(|| "Prepare delta_g2_neg"), &delta_g2_neg)?;

        Ok(PreparedVerifyingKeyGadget {
            alpha_g1_beta_g2: self.alpha_g1_beta_g2.clone(),
            gamma_g2_neg_pc,
            delta_g2_neg_pc,
            gamma_abc_g1: self.gamma_abc_g1.clone(),
        })
    }
}

#[derive(Derivative)]
#[derivative(Clone(
    bound = "P::G1Gadget: Clone, P::GTGadget: Clone, P::G1PreparedGadget: Clone, \
             P::G2PreparedGadget: Clone, "
))]
pub struct PreparedVerifyingKeyGadget<
    PairingE: PairingEngine,
    ConstraintF: Field,
    P: PairingGadget<PairingE, ConstraintF>,
> {
    pub alpha_g1_beta_g2: P::GTGadget,
    pub gamma_g2_neg_pc: P::G2PreparedGadget,
    pub delta_g2_neg_pc: P::G2PreparedGadget,
    pub gamma_abc_g1: Vec<P::G1Gadget>,
}

pub struct Groth16VerifierGadget<PairingE, ConstraintF, P>
where
    PairingE: PairingEngine,
    ConstraintF: Field,
    P: PairingGadget<PairingE, ConstraintF>,
{
    _pairing_engine: PhantomData<PairingE>,
    _engine: PhantomData<ConstraintF>,
    _pairing_gadget: PhantomData<P>,
}

impl<PairingE, ConstraintF, P, C, V> NIZKVerifierGadget<Groth16<PairingE, C, V>, ConstraintF>
    for Groth16VerifierGadget<PairingE, ConstraintF, P>
where
    PairingE: PairingEngine,
    ConstraintF: Field,
    C: ConstraintSynthesizer<PairingE::Fr>,
    V: ToConstraintField<PairingE::Fr>,
    P: PairingGadget<PairingE, ConstraintF>,
{
    type VerificationKeyGadget = VerifyingKeyGadget<PairingE, ConstraintF, P>;
    type ProofGadget = ProofGadget<PairingE, ConstraintF, P>;

    fn check_verify<'a, CS, I, T>(
        mut cs: CS,
        vk: &Self::VerificationKeyGadget,
        mut public_inputs: I,
        proof: &Self::ProofGadget,
    ) -> Result<(), SynthesisError>
    where
        CS: ConstraintSystemAbstract<ConstraintF>,
        I: Iterator<Item = &'a T>,
        T: 'a + ToBitsGadget<ConstraintF> + ?Sized,
    {
        let pvk = vk.prepare(&mut cs.ns(|| "Prepare vk"))?;

        let g_ic = {
            let mut cs = cs.ns(|| "Process input");
            let mut g_ic = pvk.gamma_abc_g1[0].clone();
            let mut input_len = 1;
            for (i, (input, b)) in public_inputs
                .by_ref()
                .zip(pvk.gamma_abc_g1.iter().skip(1))
                .enumerate()
            {
                let input_bits = input.to_bits(cs.ns(|| format!("Input {}", i)))?;
                g_ic = b
                    .mul_bits(cs.ns(|| format!("Mul {}", i)), input_bits.iter())?
                    .add(cs.ns(|| format!("Add {}", i)), &g_ic)?;
                input_len += 1;
            }
            // Check that the input and the query in the verification are of the
            // same length.
            if input_len != pvk.gamma_abc_g1.len() || public_inputs.next().is_some() {
                return Err(SynthesisError::Other(format!(
                    "Input and query must have the same length. Input len: {}, gamma_abc_g1 len: {}",
                    input_len,
                    pvk.gamma_abc_g1.len()
                )));
            }
            g_ic
        };

        let test_exp = {
            let proof_a_prep = P::prepare_g1(cs.ns(|| "Prepare proof a"), &proof.a)?;
            let proof_b_prep = P::prepare_g2(cs.ns(|| "Prepare proof b"), &proof.b)?;
            let proof_c_prep = P::prepare_g1(cs.ns(|| "Prepare proof c"), &proof.c)?;

            let g_ic_prep = P::prepare_g1(cs.ns(|| "Prepare g_ic"), &g_ic)?;

            P::miller_loop(
                cs.ns(|| "Miller loop 1"),
                &[proof_a_prep, g_ic_prep, proof_c_prep],
                &[
                    proof_b_prep,
                    pvk.gamma_g2_neg_pc.clone(),
                    pvk.delta_g2_neg_pc.clone(),
                ],
            )?
        };

        let test = P::final_exponentiation(cs.ns(|| "Final Exp"), &test_exp).unwrap();

        test.enforce_equal(cs.ns(|| "Test 1"), &pvk.alpha_g1_beta_g2)?;
        Ok(())
    }
}

impl<PairingE, ConstraintF, P> AllocGadget<VerifyingKey<PairingE>, ConstraintF>
    for VerifyingKeyGadget<PairingE, ConstraintF, P>
where
    PairingE: PairingEngine,
    ConstraintF: Field,
    P: PairingGadget<PairingE, ConstraintF>,
{
    #[inline]
    fn alloc_without_check<FN, T, CS: ConstraintSystemAbstract<ConstraintF>>(
        mut cs: CS,
        value_gen: FN,
    ) -> Result<Self, SynthesisError>
    where
        FN: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<VerifyingKey<PairingE>>,
    {
        value_gen().and_then(|vk| {
            let VerifyingKey {
                alpha_g1_beta_g2,
                gamma_g2,
                delta_g2,
                gamma_abc_g1,
            } = vk.borrow().clone();
            let alpha_g1_beta_g2 =
                P::GTGadget::alloc_without_check(cs.ns(|| "alpha_g1_beta_g2"), || {
                    Ok(alpha_g1_beta_g2)
                })?;
            let gamma_g2 = P::G2Gadget::alloc_without_check(cs.ns(|| "gamma_g2"), || {
                Ok(gamma_g2.into_projective())
            })?;
            let delta_g2 = P::G2Gadget::alloc_without_check(cs.ns(|| "delta_g2"), || {
                Ok(delta_g2.into_projective())
            })?;

            let gamma_abc_g1 = gamma_abc_g1
                .iter()
                .enumerate()
                .map(|(i, gamma_abc_i)| {
                    P::G1Gadget::alloc_without_check(cs.ns(|| format!("gamma_abc_{}", i)), || {
                        Ok(gamma_abc_i.into_projective())
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .collect::<Result<_, _>>()?;
            Ok(Self {
                alpha_g1_beta_g2,
                gamma_g2,
                delta_g2,
                gamma_abc_g1,
            })
        })
    }

    #[inline]
    fn alloc<FN, T, CS: ConstraintSystemAbstract<ConstraintF>>(
        mut cs: CS,
        value_gen: FN,
    ) -> Result<Self, SynthesisError>
    where
        FN: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<VerifyingKey<PairingE>>,
    {
        value_gen().and_then(|vk| {
            let VerifyingKey {
                alpha_g1_beta_g2,
                gamma_g2,
                delta_g2,
                gamma_abc_g1,
            } = vk.borrow().clone();
            let alpha_g1_beta_g2 =
                P::GTGadget::alloc(cs.ns(|| "alpha_g1_beta_g2"), || Ok(alpha_g1_beta_g2))
                    .and_then(|alpha_g1_beta_g2_g| {
                        let zero_g = P::GTGadget::zero(
                            cs.ns(|| "alloc zero for alpha_g1_beta_g2 comparison"),
                        )?;
                        alpha_g1_beta_g2_g.enforce_not_equal(
                            cs.ns(|| "alpha_g1_beta_g2 must not be zero"),
                            &zero_g,
                        )?;
                        Ok(alpha_g1_beta_g2_g)
                    })?;

            let gamma_g2 =
                P::G2Gadget::alloc(cs.ns(|| "gamma_g2"), || Ok(gamma_g2.into_projective()))
                    .and_then(|gamma_g2_g| {
                        gamma_g2_g
                            .is_zero(cs.ns(|| "is gamma_g2 zero"))?
                            .enforce_equal(
                                cs.ns(|| "gamma_g2 must not be zero"),
                                &Boolean::constant(false),
                            )?;
                        Ok(gamma_g2_g)
                    })?;

            let delta_g2 =
                P::G2Gadget::alloc(cs.ns(|| "delta_g2"), || Ok(delta_g2.into_projective()))
                    .and_then(|delta_g2_g| {
                        delta_g2_g
                            .is_zero(cs.ns(|| "is delta_g2 zero"))?
                            .enforce_equal(
                                cs.ns(|| "delta_g2 must not be zero"),
                                &Boolean::constant(false),
                            )?;
                        Ok(delta_g2_g)
                    })?;

            let gamma_abc_g1 = gamma_abc_g1
                .iter()
                .enumerate()
                .map(|(i, gamma_abc_i)| {
                    P::G1Gadget::alloc(cs.ns(|| format!("gamma_abc_{}", i)), || {
                        Ok(gamma_abc_i.into_projective())
                    })
                    .and_then(|input_g| {
                        input_g
                            .is_zero(cs.ns(|| format!("is input {} zero", i)))?
                            .enforce_equal(
                                cs.ns(|| format!("input {} must not be zero", i)),
                                &Boolean::constant(false),
                            )?;
                        Ok(input_g)
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .collect::<Result<_, _>>()?;
            Ok(Self {
                alpha_g1_beta_g2,
                gamma_g2,
                delta_g2,
                gamma_abc_g1,
            })
        })
    }

    #[inline]
    fn alloc_checked<FN, T, CS: ConstraintSystemAbstract<ConstraintF>>(
        mut cs: CS,
        value_gen: FN,
    ) -> Result<Self, SynthesisError>
    where
        FN: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<VerifyingKey<PairingE>>,
    {
        value_gen().and_then(|vk| {
            let VerifyingKey {
                alpha_g1_beta_g2,
                gamma_g2,
                delta_g2,
                gamma_abc_g1,
            } = vk.borrow().clone();
            let alpha_g1_beta_g2 =
                P::GTGadget::alloc_checked(cs.ns(|| "alpha_g1_beta_g2"), || Ok(alpha_g1_beta_g2))
                    .and_then(|alpha_g1_beta_g2_g| {
                    let zero_g =
                        P::GTGadget::zero(cs.ns(|| "alloc zero for alpha_g1_beta_g2 comparison"))?;
                    alpha_g1_beta_g2_g.enforce_not_equal(
                        cs.ns(|| "alpha_g1_beta_g2 must not be zero"),
                        &zero_g,
                    )?;
                    Ok(alpha_g1_beta_g2_g)
                })?;

            let gamma_g2 =
                P::G2Gadget::alloc_checked(cs.ns(|| "gamma_g2"), || Ok(gamma_g2.into_projective()))
                    .and_then(|gamma_g2_g| {
                        gamma_g2_g
                            .is_zero(cs.ns(|| "is gamma_g2 zero"))?
                            .enforce_equal(
                                cs.ns(|| "gamma_g2 must not be zero"),
                                &Boolean::constant(false),
                            )?;
                        Ok(gamma_g2_g)
                    })?;

            let delta_g2 =
                P::G2Gadget::alloc_checked(cs.ns(|| "delta_g2"), || Ok(delta_g2.into_projective()))
                    .and_then(|delta_g2_g| {
                        delta_g2_g
                            .is_zero(cs.ns(|| "is delta_g2 zero"))?
                            .enforce_equal(
                                cs.ns(|| "delta_g2 must not be zero"),
                                &Boolean::constant(false),
                            )?;
                        Ok(delta_g2_g)
                    })?;

            let gamma_abc_g1 = gamma_abc_g1
                .iter()
                .enumerate()
                .map(|(i, gamma_abc_i)| {
                    P::G1Gadget::alloc_checked(cs.ns(|| format!("gamma_abc_{}", i)), || {
                        Ok(gamma_abc_i.into_projective())
                    })
                    .and_then(|input_g| {
                        input_g
                            .is_zero(cs.ns(|| format!("is input {} zero", i)))?
                            .enforce_equal(
                                cs.ns(|| format!("input {} must not be zero", i)),
                                &Boolean::constant(false),
                            )?;
                        Ok(input_g)
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .collect::<Result<_, _>>()?;
            Ok(Self {
                alpha_g1_beta_g2,
                gamma_g2,
                delta_g2,
                gamma_abc_g1,
            })
        })
    }

    #[inline]
    fn alloc_input<FN, T, CS: ConstraintSystemAbstract<ConstraintF>>(
        mut cs: CS,
        value_gen: FN,
    ) -> Result<Self, SynthesisError>
    where
        FN: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<VerifyingKey<PairingE>>,
    {
        value_gen().and_then(|vk| {
            let VerifyingKey {
                alpha_g1_beta_g2,
                gamma_g2,
                delta_g2,
                gamma_abc_g1,
            } = vk.borrow().clone();
            let alpha_g1_beta_g2 =
                P::GTGadget::alloc(cs.ns(|| "alpha_g1_beta_g2"), || Ok(alpha_g1_beta_g2))?;
            let gamma_g2 =
                P::G2Gadget::alloc_input(cs.ns(|| "gamma_g2"), || Ok(gamma_g2.into_projective()))?;
            let delta_g2 =
                P::G2Gadget::alloc_input(cs.ns(|| "delta_g2"), || Ok(delta_g2.into_projective()))?;

            let gamma_abc_g1 = gamma_abc_g1
                .iter()
                .enumerate()
                .map(|(i, gamma_abc_i)| {
                    P::G1Gadget::alloc_input(cs.ns(|| format!("gamma_abc_{}", i)), || {
                        Ok(gamma_abc_i.into_projective())
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .collect::<Result<_, _>>()?;

            Ok(Self {
                alpha_g1_beta_g2,
                gamma_g2,
                delta_g2,
                gamma_abc_g1,
            })
        })
    }
}

impl<PairingE, ConstraintF, P> AllocGadget<Proof<PairingE>, ConstraintF>
    for ProofGadget<PairingE, ConstraintF, P>
where
    PairingE: PairingEngine,
    ConstraintF: Field,
    P: PairingGadget<PairingE, ConstraintF>,
{
    #[inline]
    fn alloc<FN, T, CS: ConstraintSystemAbstract<ConstraintF>>(
        mut cs: CS,
        value_gen: FN,
    ) -> Result<Self, SynthesisError>
    where
        FN: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<Proof<PairingE>>,
    {
        value_gen().and_then(|proof| {
            let Proof { a, b, c } = proof.borrow().clone();

            let a = P::G1Gadget::alloc_checked(cs.ns(|| "a"), || Ok(a.into_projective()))
                .and_then(|a_g| {
                    a_g.is_zero(cs.ns(|| "is a zero"))?
                        .enforce_equal(cs.ns(|| "a must not be zero"), &Boolean::constant(false))?;
                    Ok(a_g)
                })?;

            let b = P::G2Gadget::alloc_checked(cs.ns(|| "b"), || Ok(b.into_projective()))
                .and_then(|b_g| {
                    b_g.is_zero(cs.ns(|| "is b zero"))?
                        .enforce_equal(cs.ns(|| "b must not be zero"), &Boolean::constant(false))?;
                    Ok(b_g)
                })?;

            let c = P::G1Gadget::alloc_checked(cs.ns(|| "c"), || Ok(c.into_projective()))
                .and_then(|c_g| {
                    c_g.is_zero(cs.ns(|| "is c zero"))?
                        .enforce_equal(cs.ns(|| "c must not be zero"), &Boolean::constant(false))?;
                    Ok(c_g)
                })?;

            Ok(Self { a, b, c })
        })
    }

    #[inline]
    fn alloc_input<FN, T, CS: ConstraintSystemAbstract<ConstraintF>>(
        mut cs: CS,
        value_gen: FN,
    ) -> Result<Self, SynthesisError>
    where
        FN: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<Proof<PairingE>>,
    {
        value_gen().and_then(|proof| {
            let Proof { a, b, c } = proof.borrow().clone();
            // We don't need to check here because the prime order check can be performed
            // in plain.
            let a = P::G1Gadget::alloc_input(cs.ns(|| "a"), || Ok(a.into_projective()))?;
            let b = P::G2Gadget::alloc_input(cs.ns(|| "b"), || Ok(b.into_projective()))?;
            let c = P::G1Gadget::alloc_input(cs.ns(|| "c"), || Ok(c.into_projective()))?;
            Ok(Self { a, b, c })
        })
    }
}

impl<PairingE, ConstraintF, P> ToBytesGadget<ConstraintF>
    for VerifyingKeyGadget<PairingE, ConstraintF, P>
where
    PairingE: PairingEngine,
    ConstraintF: Field,
    P: PairingGadget<PairingE, ConstraintF>,
{
    #[inline]
    fn to_bytes<CS: ConstraintSystemAbstract<ConstraintF>>(
        &self,
        mut cs: CS,
    ) -> Result<Vec<UInt8>, SynthesisError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(
            &self
                .alpha_g1_beta_g2
                .to_bytes(&mut cs.ns(|| "alpha_g1_beta_g2 to bytes"))?,
        );
        bytes.extend_from_slice(&self.gamma_g2.to_bytes(&mut cs.ns(|| "gamma_g2 to bytes"))?);
        bytes.extend_from_slice(&self.delta_g2.to_bytes(&mut cs.ns(|| "delta_g2 to bytes"))?);
        for (i, g) in self.gamma_abc_g1.iter().enumerate() {
            let mut cs = cs.ns(|| format!("Iteration {}", i));
            bytes.extend_from_slice(&g.to_bytes(&mut cs.ns(|| "g"))?);
        }
        Ok(bytes)
    }

    fn to_bytes_strict<CS: ConstraintSystemAbstract<ConstraintF>>(
        &self,
        mut cs: CS,
    ) -> Result<Vec<UInt8>, SynthesisError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(
            &self
                .alpha_g1_beta_g2
                .to_bytes_strict(&mut cs.ns(|| "alpha_g1_beta_g2 to bytes"))?,
        );
        bytes.extend_from_slice(
            &self
                .gamma_g2
                .to_bytes_strict(&mut cs.ns(|| "gamma_g2 to bytes"))?,
        );
        bytes.extend_from_slice(
            &self
                .delta_g2
                .to_bytes_strict(&mut cs.ns(|| "delta_g2 to bytes"))?,
        );
        for (i, g) in self.gamma_abc_g1.iter().enumerate() {
            let mut cs = cs.ns(|| format!("Iteration {}", i));
            bytes.extend_from_slice(&g.to_bytes_strict(&mut cs.ns(|| "g"))?);
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod test {
    // Note: For MNT4 and MNT6, num_inputs has been set to 2 otherwise, when executing the
    // tests simultaneously from the cargo test framework, the memory will run out and the
    // tests execution will crash.
    use proof_systems::groth16::*;
    use r1cs_core::{
        ConstraintSynthesizer, ConstraintSystem, ConstraintSystemAbstract,
        ConstraintSystemDebugger, SynthesisError, SynthesisMode,
    };

    use super::*;
    use algebra::{ToBits, UniformRand};
    use r1cs_std::boolean::Boolean;
    use rand::thread_rng;

    struct Bench<F: Field> {
        inputs: Vec<Option<F>>,
        num_constraints: usize,
    }

    impl<F: Field> ConstraintSynthesizer<F> for Bench<F> {
        fn generate_constraints<CS: ConstraintSystemAbstract<F>>(
            self,
            cs: &mut CS,
        ) -> Result<(), SynthesisError> {
            assert!(self.inputs.len() >= 2);
            assert!(self.num_constraints >= self.inputs.len());

            let mut variables: Vec<_> = Vec::with_capacity(self.inputs.len());
            for (i, input) in self.inputs.into_iter().enumerate() {
                let input_var = cs.alloc_input(
                    || format!("Input {}", i),
                    || input.ok_or(SynthesisError::AssignmentMissing),
                )?;
                variables.push((input, input_var));
            }

            for i in 0..self.num_constraints {
                let new_entry = {
                    let (input_1_val, input_1_var) = variables[i];
                    let (input_2_val, input_2_var) = variables[i + 1];
                    let result_val = input_1_val
                        .and_then(|input_1| input_2_val.map(|input_2| input_1 * &input_2));
                    let result_var = cs.alloc(
                        || format!("Result {}", i),
                        || result_val.ok_or(SynthesisError::AssignmentMissing),
                    )?;
                    cs.enforce(
                        || format!("Enforce constraint {}", i),
                        |lc| lc + input_1_var,
                        |lc| lc + input_2_var,
                        |lc| lc + result_var,
                    );
                    (result_val, result_var)
                };
                variables.push(new_entry);
            }
            Ok(())
        }
    }

    fn groth16_verifier_test<E: PairingEngine, PG: PairingGadget<E, E::Fq>>() {
        let num_inputs = 2;
        let num_constraints = 100;
        let rng = &mut thread_rng();
        let mut inputs: Vec<Option<E::Fr>> = Vec::with_capacity(num_inputs);
        for _ in 0..num_inputs {
            inputs.push(Some(E::Fr::rand(rng)));
        }
        let params = {
            let c = Bench::<E::Fr> {
                inputs: vec![None; num_inputs],
                num_constraints,
            };

            generate_random_parameters(c, rng).unwrap()
        };

        {
            let proof = {
                // Create an instance of our circuit (with the
                // witness)
                let c = Bench {
                    inputs: inputs.clone(),
                    num_constraints,
                };
                // Create a groth16 proof with our parameters.
                create_random_proof(c, &params, rng).unwrap()
            };

            // assert!(!verify_proof(&pvk, &proof, &[a]).unwrap());
            let mut cs = ConstraintSystem::<E::Fq>::new(SynthesisMode::Debug);

            let inputs: Vec<_> = inputs.into_iter().map(|input| input.unwrap()).collect();
            let mut input_gadgets = Vec::new();

            {
                let mut cs = cs.ns(|| "Allocate Input");
                for (i, input) in inputs.into_iter().enumerate() {
                    let mut input_bits = input.write_bits();
                    // Input must be in little-endian, but BitIterator outputs in big-endian.
                    input_bits.reverse();

                    let input_bits = Boolean::alloc_input_vec(
                        cs.ns(|| format!("Input {}", i)),
                        input_bits.as_slice(),
                    )
                    .unwrap();
                    input_gadgets.push(input_bits);
                }
            }

            let vk_gadget =
                VerifyingKeyGadget::<E, E::Fq, PG>::alloc_input(cs.ns(|| "Vk"), || Ok(&params.vk))
                    .unwrap();

            let proof_gadget =
                ProofGadget::<E, E::Fq, PG>::alloc(cs.ns(|| "Proof"), || Ok(proof.clone()))
                    .unwrap();

            println!("Time to verify!\n\n\n\n");
            <Groth16VerifierGadget<E, E::Fq, PG> as NIZKVerifierGadget<
                Groth16<E, Bench<E::Fr>, E::Fr>,
                E::Fq,
            >>::check_verify(
                cs.ns(|| "Verify"),
                &vk_gadget,
                input_gadgets.iter(),
                &proof_gadget,
            )
            .unwrap();
            if !cs.is_satisfied() {
                println!("=========================================================");
                println!("Unsatisfied constraints:");
                println!("{:?}", cs.which_is_unsatisfied().unwrap());
                println!("=========================================================");
            }

            // cs.print_named_objects();
            assert!(cs.is_satisfied());
        }
    }

    #[test]
    fn bls12_377_groth16_verifier_test() {
        use algebra::curves::bls12_377::Bls12_377;
        use r1cs_std::instantiated::bls12_377::PairingGadget;

        groth16_verifier_test::<Bls12_377, PairingGadget>();
    }

    #[test]
    fn mnt4_753_groth16_verifier_test() {
        use algebra::curves::mnt4753::MNT4;
        use r1cs_std::instantiated::mnt4_753::PairingGadget;

        groth16_verifier_test::<MNT4, PairingGadget>();
    }

    #[ignore]
    #[test]
    fn mnt6_753_groth16_verifier_test() {
        use algebra::curves::mnt6753::MNT6;
        use r1cs_std::instantiated::mnt6_753::PairingGadget;

        groth16_verifier_test::<MNT6, PairingGadget>();
    }

    #[test]
    fn bn_382_groth16_verifier_test() {
        use algebra::curves::bn_382::Bn382;
        use r1cs_std::instantiated::bn_382::PairingGadget;

        groth16_verifier_test::<Bn382, PairingGadget>();
    }
}
