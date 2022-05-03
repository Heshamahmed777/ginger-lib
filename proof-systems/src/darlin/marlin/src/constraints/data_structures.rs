//! The data structures for Coboundary Marlin verifier gadget:
//!     - verifier key gadget,
//!     - SNARK proof gadget,
use crate::iop::indexer::*;
use crate::{Proof, Vec, VerifierKey};
use algebra::EndoMulCurve;
use poly_commit::constraints::PolynomialCommitmentVerifierGadget;
use poly_commit::PolynomialCommitment;
use r1cs_core::{ConstraintSystemAbstract, SynthesisError};
use r1cs_std::alloc::AllocGadget;
use r1cs_std::fields::fp::FpGadget;
use r1cs_std::fields::nonnative::nonnative_field_gadget::NonNativeFieldGadget;
use r1cs_std::to_field_gadget_vec::ToConstraintFieldGadget;
use r1cs_std::uint8::UInt8;
use std::borrow::Borrow;

/// Gadget for verification key for a specific R1CS.
pub struct VerifierKeyGadget<
    G: EndoMulCurve,
    PC: PolynomialCommitment<G>,
    PCG: PolynomialCommitmentVerifierGadget<G::BaseField, G, PC>,
> {
    /// Stores R1CS metrics as usually supplied by the constraint system.
    pub index_info: IndexInfo<G::ScalarField>,
    /// Commitments to the indexed polynomials.
    pub index_comms: Vec<PCG::CommitmentGadget>,
    /// Commitments to the Lagrange polynomials over the input domain.
    pub lagrange_comms: Vec<PC::Commitment>,
    /// Hash of the above elements
    pub vk_hash: Vec<UInt8>,
}

impl<G, PC, PCG> VerifierKeyGadget<G, PC, PCG>
where
    G: EndoMulCurve,
    PC: PolynomialCommitment<G>,
    PCG: PolynomialCommitmentVerifierGadget<G::BaseField, G, PC>,
{
    /// Iterate over the commitments to indexed polynomials in `self`.
    pub fn iter(&self) -> impl Iterator<Item = &PCG::CommitmentGadget> {
        self.index_comms.iter()
    }

    /// Return hash of this key
    pub fn get_hash(&self) -> &[UInt8] {
        &self.vk_hash
    }
}

impl<G, PC, PCG> AllocGadget<VerifierKey<G, PC>, G::BaseField> for VerifierKeyGadget<G, PC, PCG>
where
    G: EndoMulCurve,
    PC: PolynomialCommitment<G>,
    PCG: PolynomialCommitmentVerifierGadget<G::BaseField, G, PC>,
{
    fn alloc<F, T, CS: ConstraintSystemAbstract<G::BaseField>>(
        mut cs: CS,
        f: F,
    ) -> Result<Self, SynthesisError>
    where
        F: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<VerifierKey<G, PC>>,
    {
        let t = f()?;
        let VerifierKey {
            index_info,
            index_comms,
            lagrange_comms,
            vk_hash,
        } = t.borrow();

        let index_comms_gadget = index_comms
            .iter()
            .enumerate()
            .map(|(i, comm)| {
                PCG::CommitmentGadget::alloc_without_check::<_, PC::Commitment, _>(
                    cs.ns(|| format!("alloc index commitment {}", i)),
                    || Ok(comm.clone()),
                )
                .unwrap()
            })
            .collect();
        let vk_hash_gadget = UInt8::alloc_vec(cs.ns(|| "alloc vk hash"), vk_hash)?;

        Ok(Self {
            index_info: *index_info,
            index_comms: index_comms_gadget,
            lagrange_comms: lagrange_comms.clone(),
            vk_hash: vk_hash_gadget,
        })
    }

    fn alloc_input<F, T, CS: ConstraintSystemAbstract<G::BaseField>>(
        mut cs: CS,
        f: F,
    ) -> Result<Self, SynthesisError>
    where
        F: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<VerifierKey<G, PC>>,
    {
        let t = f()?;
        let VerifierKey {
            index_info,
            index_comms,
            lagrange_comms,
            vk_hash,
        } = t.borrow();

        let index_comms_gadget = index_comms
            .iter()
            .enumerate()
            .map(|(i, comm)| {
                PCG::CommitmentGadget::alloc_input::<_, PC::Commitment, _>(
                    cs.ns(|| format!("alloc index commitment {}", i)),
                    || Ok(comm.clone()),
                )
                .unwrap()
            })
            .collect();
        let vk_hash_gadget = UInt8::alloc_input_vec(cs.ns(|| "alloc vk hash"), vk_hash)?;

        Ok(Self {
            index_info: *index_info,
            index_comms: index_comms_gadget,
            lagrange_comms: lagrange_comms.clone(),
            vk_hash: vk_hash_gadget,
        })
    }
}

impl<G, PC, PCG> ToConstraintFieldGadget<G::BaseField> for VerifierKeyGadget<G, PC, PCG>
where
    G: EndoMulCurve,
    PC: PolynomialCommitment<G>,
    PCG: PolynomialCommitmentVerifierGadget<G::BaseField, G, PC>,
{
    type FieldGadget = FpGadget<G::BaseField>;

    fn to_field_gadget_elements<CS: ConstraintSystemAbstract<G::BaseField>>(
        &self,
        mut cs: CS,
    ) -> Result<Vec<Self::FieldGadget>, SynthesisError> {
        // For our purposes it's good enough to convert only the vk digest to field elements
        self
            .vk_hash
            .as_slice()
            .to_field_gadget_elements(cs.ns(|| "vk digest to fes"))
    }
}

/// Gadget for the SNARK proof.
pub struct ProofGadget<
    G: EndoMulCurve,
    PC: PolynomialCommitment<G>,
    PCG: PolynomialCommitmentVerifierGadget<G::BaseField, G, PC>,
> {
    /// Commitments to the polynomials produced by the prover
    pub commitments: Vec<Vec<PCG::CommitmentGadget>>,
    /// Evaluations of these polynomials.
    pub evaluations: Vec<NonNativeFieldGadget<G::ScalarField, G::BaseField>>,
    /// A batch evaluation proof from the polynomial commitment.
    pub pc_proof: PCG::MultiPointProofGadget,
}

impl<G, PC, PCG> AllocGadget<Proof<G, PC>, G::BaseField> for ProofGadget<G, PC, PCG>
where
    G: EndoMulCurve,
    PC: PolynomialCommitment<G>,
    PCG: PolynomialCommitmentVerifierGadget<G::BaseField, G, PC>,
{
    fn alloc<F, T, CS: ConstraintSystemAbstract<G::BaseField>>(
        mut cs: CS,
        f: F,
    ) -> Result<Self, SynthesisError>
    where
        F: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<Proof<G, PC>>,
    {
        let t = f()?;
        let Proof {
            commitments,
            evaluations,
            pc_proof,
            ..
        } = t.borrow();

        let commitment_gadgets = commitments
            .iter()
            .enumerate()
            .map(|(i, lst)| {
                lst.iter()
                    .enumerate()
                    .map(|(j, comm)| {
                        PCG::CommitmentGadget::alloc_checked::<_, PC::Commitment, _>(
                            cs.ns(|| format!("alloc commitment {}{}", i, j)),
                            || Ok(comm.clone()),
                        )
                        .unwrap()
                    })
                    .collect()
            })
            .collect();

        let evaluation_gadgets = evaluations
            .iter()
            .enumerate()
            .map(|(i, eval)| {
                NonNativeFieldGadget::alloc(cs.ns(|| format!("alloc evaluation {}", i)), || {
                    Ok(eval)
                })
                .unwrap()
            })
            .collect();

        let pc_proof =
            PCG::MultiPointProofGadget::alloc(cs.ns(|| "alloc multipoint PC proof"), || {
                Ok(pc_proof)
            })?;

        Ok(Self {
            commitments: commitment_gadgets,
            evaluations: evaluation_gadgets,
            pc_proof,
        })
    }

    fn alloc_input<F, T, CS: ConstraintSystemAbstract<G::BaseField>>(
        mut cs: CS,
        f: F,
    ) -> Result<Self, SynthesisError>
    where
        F: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<Proof<G, PC>>,
    {
        let t = f()?;
        let Proof {
            commitments,
            evaluations,
            pc_proof,
            ..
        } = t.borrow();

        let commitment_gadgets = commitments
            .iter()
            .enumerate()
            .map(|(i, lst)| {
                lst.iter()
                    .enumerate()
                    .map(|(j, comm)| {
                        PCG::CommitmentGadget::alloc_input::<_, PC::Commitment, _>(
                            cs.ns(|| format!("alloc commitment {}{}", i, j)),
                            || Ok(comm.clone()),
                        )
                        .unwrap()
                    })
                    .collect()
            })
            .collect();

        let evaluation_gadgets = evaluations
            .iter()
            .enumerate()
            .map(|(i, eval)| {
                NonNativeFieldGadget::alloc_input(
                    cs.ns(|| format!("alloc evaluation {}", i)),
                    || Ok(eval),
                )
                .unwrap()
            })
            .collect();

        let pc_proof =
            PCG::MultiPointProofGadget::alloc_input(cs.ns(|| "alloc multipoint PC proof"), || {
                Ok(pc_proof)
            })?;

        Ok(Self {
            commitments: commitment_gadgets,
            evaluations: evaluation_gadgets,
            pc_proof,
        })
    }
}