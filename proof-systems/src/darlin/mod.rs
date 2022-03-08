//! The module for our Darlin proof carrying data (PCD) scheme as described in
//! our [DarlinProofTree doc](TODO: link).
//! The Darlin PCD scheme is based on (a variant of) Marlin/dlog, aggregating
//! the dlog "hard parts" as well as the inner sumchecks across multiple
//! circuits.
//! For now the module serves only basic structs and functions for the final
//! nodes of our conversion/exiting chain (which is either a "Simple Marlin",
//! or a "Final Darlin"). It is split into the following submodules
//!     - `accumulators`: accumulator structs and their aggregation schemes as
//!     stand-alone non-interactive arguments. Although the stand-alone NI arguments
//!     are not applied in recursion, they are useful for post-prossing.
//!     - `pcd`: Proof carrying data from the verifier point of view.
//!     - `proof_aggregator`: utilities for proof post-processing, such as batch
//!     verification and aggregation of their dlog hard parts.
pub mod accumulators;
pub mod data_structures;
pub mod error;
pub mod pcd;
pub mod proof_aggregator;

pub mod tests;

use crate::darlin::{
    data_structures::*,
    error::FinalDarlinError,
    pcd::{
        final_darlin::{FinalDarlinPCD, FinalDarlinPCDVerifierKey},
        simple_marlin::MarlinProof,
        PCDCircuit, PCD,
    },
};
use algebra::{EndoMulCurve, Group, GroupVec, ToConstraintField};
use digest::Digest;
use fiat_shamir::FiatShamirRng;
use marlin::{Marlin, ProverKey as MarlinProverKey, VerifierKey as MarlinVerifierKey};
use poly_commit::{
    ipa_pc::{
        CommitterKey as DLogProverKey, InnerProductArgPC, Parameters,
        VerifierKey as DLogVerifierKey,
    },
    DomainExtendedPolynomialCommitment, Evaluations, LabeledCommitment, QuerySet,
};
use rand::RngCore;
use std::marker::PhantomData;

/// FinalDarlin proof system. It is simply a (coboundary) Marlin SNARK of a dedicated
/// recursive `PCDCircuit`.
pub type FinalDarlinProverKey<G, PC> = MarlinProverKey<G, PC>;
pub type FinalDarlinVerifierKey<G, PC> = MarlinVerifierKey<G, PC>;

// A final Darlin in G1, and the previous node in G2.
pub struct FinalDarlin<'a, G1: EndoMulCurve, G2: EndoMulCurve, FS: FiatShamirRng + 'static>(
    #[doc(hidden)] PhantomData<G1>,
    #[doc(hidden)] PhantomData<G2>,
    #[doc(hidden)] PhantomData<FS>,
    #[doc(hidden)] PhantomData<&'a ()>,
);

impl<'a, G1, G2, FS> FinalDarlin<'a, G1, G2, FS>
where
    G1: EndoMulCurve<BaseField = <G2 as Group>::ScalarField>
        + ToConstraintField<<G2 as Group>::ScalarField>,
    G2: EndoMulCurve<BaseField = <G1 as Group>::ScalarField>
        + ToConstraintField<<G1 as Group>::ScalarField>,
    FS: FiatShamirRng + 'static,
{
    /// Generate the universal prover and verifier keys for Marlin.
    pub fn universal_setup<D: Digest>(
        num_constraints: usize,
        num_variables: usize,
        num_non_zero: usize,
        zk: bool,
    ) -> Result<(Parameters<G1>, Parameters<G2>), FinalDarlinError> {
        let srs_g1 = Marlin::<
            G1,
            DomainExtendedPolynomialCommitment<G1, InnerProductArgPC<G1, FS>>,
        >::universal_setup::<D>(num_constraints, num_variables, num_non_zero, zk)?;

        let srs_g2 = Marlin::<
            G2,
            DomainExtendedPolynomialCommitment<G2, InnerProductArgPC<G2, FS>>,
        >::universal_setup::<D>(num_constraints, num_variables, num_non_zero, zk)?;

        Ok((srs_g1, srs_g2))
    }

    /// Generate the index-specific (i.e., circuit-specific) prover and verifier
    /// keys from the dedicated PCDCircuit.
    /// This is a deterministic algorithm that anyone can rerun.
    pub fn index<C: PCDCircuit<G1>, D: Digest>(
        committer_key: &DLogProverKey<G1>,
        config: C::SetupData,
    ) -> Result<
        (
            FinalDarlinProverKey<
                G1,
                DomainExtendedPolynomialCommitment<G1, InnerProductArgPC<G1, FS>>,
            >,
            FinalDarlinVerifierKey<
                G1,
                DomainExtendedPolynomialCommitment<G1, InnerProductArgPC<G1, FS>>,
            >,
        ),
        FinalDarlinError,
    > {
        let c = C::init(config);
        let res = Marlin::<G1, DomainExtendedPolynomialCommitment<G1, InnerProductArgPC<G1, FS>>>::circuit_specific_setup::<_, D>(committer_key, c)?;

        Ok(res)
    }

    /// Create and return a FinalDarlinPCD, given previous PCDs and a PCDCircuit
    /// that (partially) verify them along with some additional data.
    pub fn prove<C>(
        index_pk: &FinalDarlinProverKey<
            G1,
            DomainExtendedPolynomialCommitment<G1, InnerProductArgPC<G1, FS>>,
        >,
        pc_pk: &DLogProverKey<G1>,
        config: C::SetupData,
        // In future, this will be explicitly a RainbowDarlinPCD
        previous: Vec<C::PreviousPCD>,
        previous_vks: Vec<<C::PreviousPCD as PCD>::PCDVerifierKey>,
        additional_data: C::AdditionalData,
        zk: bool,
        zk_rng: Option<&mut dyn RngCore>,
    ) -> Result<FinalDarlinPCD<'a, G1, G2, FS>, FinalDarlinError>
    where
        C: PCDCircuit<G1, SystemInputs = FinalDarlinDeferredData<G1, G2>>,
    {
        // init the recursive circuit using the previous PCDs and the additional data.
        let c = C::init_state(config, previous, previous_vks, additional_data);

        // get the system and user inputs from the recursive circuit
        let sys_ins = c.get_sys_ins()?.clone();

        let usr_ins = c.get_usr_ins()?;

        // run the Marlin prover on the initialized recursive circuit
        let proof =
            Marlin::<G1, DomainExtendedPolynomialCommitment<G1, InnerProductArgPC<G1, FS>>>::prove(
                index_pk, pc_pk, c, zk, zk_rng,
            )?;

        let proof = FinalDarlinProof::<G1, G2, FS> {
            proof: MarlinProof(proof),
            deferred: sys_ins,
        };
        // let usr_ins = usr_ins.to_field_elements().map_err(|_| {
        //     FinalDarlinError::Other("Failed to convert usr ins to field elements".to_owned())
        // })?;

        Ok(FinalDarlinPCD::<G1, G2, FS>::new(proof, usr_ins))
    }

    /// Fully verify a `FinalDarlinProof` from the PCDCircuit `C`, using the PCD implementation for
    /// the FinalDarlinPCD.
    pub fn verify<R: RngCore>(
        index_vk: &FinalDarlinVerifierKey<
            G1,
            DomainExtendedPolynomialCommitment<G1, InnerProductArgPC<G1, FS>>,
        >,
        pc_vk_g1: &DLogVerifierKey<G1>,
        pc_vk_g2: &DLogVerifierKey<G2>,
        usr_ins: &[G1::ScalarField],
        proof: &FinalDarlinProof<G1, G2, FS>,
        rng: &mut R,
    ) -> Result<bool, FinalDarlinError> {
        let final_darlin_pcd = FinalDarlinPCD::<G1, G2, FS>::new(proof.clone(), usr_ins.to_vec());

        let final_darlin_pcd_vk = FinalDarlinPCDVerifierKey::<G1, G2, FS> {
            final_darlin_vk: index_vk,
            dlog_vks: (pc_vk_g1, pc_vk_g2),
        };

        let res = final_darlin_pcd.verify(&final_darlin_pcd_vk, rng)?;

        Ok(res)
    }

    /// Verifies only the IOP part of a `FinalDarlinProof`, i.e. a Marlin AHP
    /// for the PCDCircuit with correctly combined system and user inputs.
    pub fn verify_ahp(
        pc_vk: &DLogVerifierKey<G1>,
        index_vk: &FinalDarlinVerifierKey<
            G1,
            DomainExtendedPolynomialCommitment<G1, InnerProductArgPC<G1, FS>>,
        >,
        usr_ins: &[G1::ScalarField],
        proof: &FinalDarlinProof<G1, G2, FS>,
    ) -> Result<
        (
            QuerySet<'a, G1::ScalarField>,
            Evaluations<'a, G1::ScalarField>,
            Vec<LabeledCommitment<GroupVec<G1>>>,
            FS,
        ),
        FinalDarlinError,
    > {
        // Get "system inputs"
        let mut public_inputs = proof.deferred.to_field_elements().map_err(|_| {
            FinalDarlinError::Other(
                "Unable to convert proof.deferred to native field elements".to_owned(),
            )
        })?;

        // Append user inputs
        public_inputs.extend_from_slice(usr_ins);

        // Verify AHP
        let res = Marlin::<G1, DomainExtendedPolynomialCommitment<G1, InnerProductArgPC<G1, FS>>>::verify_iop(
            pc_vk, index_vk, public_inputs.as_slice(), &proof.proof
        )?;

        Ok(res)
    }

    /// Verifies the dlog open part of a `FinalDarlinProof`. This also checks the
    /// "hard part" of the opening proof.
    pub fn verify_opening(
        pc_vk: &DLogVerifierKey<G1>,
        proof: &FinalDarlinProof<G1, G2, FS>,
        labeled_comms: Vec<LabeledCommitment<GroupVec<G1>>>,
        query_set: QuerySet<'a, G1::ScalarField>,
        evaluations: Evaluations<'a, G1::ScalarField>,
        fs_rng: &mut FS,
    ) -> Result<bool, FinalDarlinError> {
        let res = Marlin::<G1, DomainExtendedPolynomialCommitment<G1, InnerProductArgPC<G1, FS>>>::verify_opening(
            pc_vk, &proof.proof, labeled_comms, query_set, evaluations, fs_rng
        )?;

        Ok(res)
    }
}
