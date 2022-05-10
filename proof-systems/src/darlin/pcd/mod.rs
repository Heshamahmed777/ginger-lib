//! Proof carrying data from accumulator SNARKS. For now, it includes only the
//! following basic elements:
//!     - trait for recursive circuits,
//!     - verifier trait for proof carrying data, and their implementation
//!     for SimpleMarlin and FinalDarlin PCDs.
use crate::darlin::{
    accumulators::{
        dlog::{DualDLogAccumulator, DualDLogItem},
        Accumulator,
    },
    data_structures::FinalDarlinDeferredData,
    pcd::{
        error::PCDError,
        final_darlin::{FinalDarlinPCD, FinalDarlinPCDVerifierKey},
        simple_marlin::{SimpleMarlinPCD, SimpleMarlinPCDVerifierKey},
    },
};
use algebra::{DualCycle, ToConstraintField, UniformRand};
use derivative::Derivative;
use fiat_shamir::FiatShamirRng;
use poly_commit::{
    ipa_pc::{CommitterKey as DLogCommitterKey, IPACurve, VerifierKey as DLogVerifierKey},
    Error as PCError, PCKey,
};
use r1cs_core::ConstraintSynthesizer;
use rand::RngCore;
use std::fmt::Debug;

pub mod error;
pub mod final_darlin;
pub mod simple_marlin;

/// Configuration parameters for the PCD scheme: for now, just the size of the
/// committer key to be used throughout the PCD scheme.
pub struct PCDParameters {
    pub segment_size: usize,
}

impl PCDParameters {
    /// We assume the DLOG keys to be generated outside the PCD scheme,
    /// so this function actually just trim them to the segment size
    /// specified in the config.
    pub fn universal_setup<G: IPACurve>(
        &self,
        params: (&DLogCommitterKey<G>, &DLogVerifierKey<G>),
    ) -> Result<(DLogCommitterKey<G>, DLogVerifierKey<G>), PCError> {
        Ok((
            params.0.trim(self.segment_size - 1)?,
            params.1.trim(self.segment_size - 1)?,
        ))
    }
}

/// Trait for the recursive circuit of a PCD node in G. Both witnesses and public inputs
/// are derived from previous proofs (PCDs) and some additional data ("payload").
/// A recursive circuit comes with a universal circuit interface, comprised of
///     - `user inputs` (i.e. the proof "statement") and
///     - `system inputs`, which is the data due to amortization and split verification,
///     aka deferred checks.
/// The additional data is used only by dedicated circuits such as a base proofs or
/// a finalizing block proofs. For the ordinary merger nodes, it is simply `None`.
pub trait PCDCircuit<G: IPACurve>: ConstraintSynthesizer<G::ScalarField> {
    /// Any data that may be needed to bootstrap the circuit that is not covered by the other
    /// fields.
    type SetupData: Clone;

    /// Additional data to be processed by the circuit.
    /// This might be related to recursion (incremental "payload"). In our PCD it is  
    /// supplementary witness data to serve additional business logic of the circuit.
    type AdditionalData;

    /// Elements that are deferred during recursion. The are derived from the PCDs
    /// passed by the nodes "below"
    type SystemInputs: ToConstraintField<G::ScalarField> + Debug + Clone;

    /// PCD type the circuit needs to verify
    type PreviousPCD: PCD;

    /// Initialize the circuit state without explicitly assigning inputs and witnesses.
    /// To be used to generate pk and vk.
    fn init(config: Self::SetupData) -> Self;

    /// Assign a concrete state to the circuit, using previous proofs and some "payload".
    /// As the circuit needs to verify previous proofs, it also needs the corresponding vks;
    fn init_state(
        config: Self::SetupData,
        previous_proofs_data: Vec<Self::PreviousPCD>,
        previous_proofs_vks: Vec<<Self::PreviousPCD as PCD>::PCDVerifierKey>,
        additional_data: Self::AdditionalData,
    ) -> Self;

    /// Extract the system inputs from a concrete instantiation of the circuit.
    /// Return Error if it's not possible to derive SystemInputs.
    fn get_sys_ins(&self) -> Result<&Self::SystemInputs, PCDError>;

    /// Extract the user inputs from a concrete instantiation of the circuit.
    /// Return Error if it's not possible to derive UserInputs.
    fn get_usr_ins(&self) -> Result<Vec<G::ScalarField>, PCDError>;

    // TODO: Think about having an additional get_circuit_inputs() function if
    //       the two above don't turn out to be flexible enough for our applications.
}

/// This trait expresses the verifier for proof carrying data from accumulator SNARKs.
/// The PCD is assumed to process a set of proof carrying data consisting of
///     - a statement,
///     - accumulator SNARK proof (i.e. a SNARK proof plus its accumulator)
pub trait PCD: Sized + Send + Sync {
    type PCDAccumulator: Accumulator;
    type PCDVerifierKey: AsRef<<Self::PCDAccumulator as Accumulator>::VerifierKey>;

    /// Perform only the efficient part (i.e. sublinear w.r.t. the circuit size) of proof verification.
    /// Typically includes few algebraic operations, e.g. the verification of Marlin's sumcheck
    /// equations, batching commitments and their claimed openings, dlog reduction,and so on.
    /// Return the accumulator for the proof if verification was successful,
    /// Error otherwise.
    fn succinct_verify(
        &self,
        vk: &Self::PCDVerifierKey,
    ) -> Result<<Self::PCDAccumulator as Accumulator>::Item, PCDError>;

    /// Perform the non-efficient part of proof verification.
    /// Verify / decide the current accumulator, by checking the non-efficient predicate.
    /// Typically involves one or several MSMs.
    fn hard_verify<R: RngCore>(
        &self,
        acc: <Self::PCDAccumulator as Accumulator>::Item,
        vk: &Self::PCDVerifierKey,
        rng: &mut R,
    ) -> Result<bool, PCDError> {
        <Self::PCDAccumulator as Accumulator>::check_items::<R>(vk.as_ref(), &[acc], rng)
            .map_err(|e| PCDError::FailedHardVerification(e.to_string()))
    }

    /// Perform full verification of `self`, i.e. both succinct and hard part.
    fn verify<R: RngCore>(&self, vk: &Self::PCDVerifierKey, rng: &mut R) -> Result<bool, PCDError> {
        let acc = self.succinct_verify(vk)?;
        self.hard_verify::<R>(acc, vk, rng)
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
/// Achieve polymorphism for PCD via an enumerable. This provides nice APIs for
/// the proof aggregation implementation and testing.
pub enum GeneralPCD<'a, G1: IPACurve, G2: IPACurve, FS: FiatShamirRng + 'static> {
    SimpleMarlin(SimpleMarlinPCD<'a, G1, FS>),
    FinalDarlin(FinalDarlinPCD<'a, G1, G2, FS>),
}

// Testing functions
impl<'a, G1, G2, FS> GeneralPCD<'a, G1, G2, FS>
where
    G1: IPACurve,
    G2: IPACurve,
    G1: DualCycle<G2>,
    FS: FiatShamirRng,
{
    pub fn randomize_usr_ins<R: RngCore>(&mut self, rng: &mut R) {
        match self {
            Self::SimpleMarlin(simple_marlin) => {
                // No sys ins (for now) for SimpleMarlin, so modify the usr_ins instead
                let ins_len = simple_marlin.usr_ins.len();
                simple_marlin.usr_ins = (0..ins_len).map(|_| G1::ScalarField::rand(rng)).collect();
            }
            Self::FinalDarlin(final_darlin) => {
                let ins_len = final_darlin.usr_ins.len();
                final_darlin.usr_ins = (0..ins_len).map(|_| G1::ScalarField::rand(rng)).collect();
            }
        }
    }

    pub fn randomize_sys_ins<R: RngCore>(
        &mut self,
        ck_g1: &DLogCommitterKey<G1>,
        ck_g2: &DLogCommitterKey<G2>,
        rng: &mut R,
    ) {
        match self {
            Self::SimpleMarlin(simple_marlin) => {
                // No sys ins (for now) for SimpleMarlin, so modify the usr_ins instead
                let ins_len = simple_marlin.usr_ins.len();
                simple_marlin.usr_ins = (0..ins_len).map(|_| G1::ScalarField::rand(rng)).collect();
            }
            Self::FinalDarlin(final_darlin) => {
                final_darlin.final_darlin_proof.deferred =
                    FinalDarlinDeferredData::<G1, G2>::generate_random::<R, FS>(rng, ck_g1, ck_g2);
            }
        }
    }
}

/// We can re-use the FinalDarlinPCDVerifierKey for GeneralPCD as it contains both
/// committer keys, and a CoboundaryMarlin and FinalDarlinProof are both verifiable
/// with a standard Marlin Verifier key. Let's introduce a new type just to be clean.
pub type DualPCDVerifierKey<'a, G1, G2, FS> = FinalDarlinPCDVerifierKey<'a, G1, G2, FS>;

impl<'a, G1, G2, FS> PCD for GeneralPCD<'a, G1, G2, FS>
where
    G1: IPACurve,
    G2: IPACurve,
    G1: DualCycle<G2>,
    FS: FiatShamirRng + 'static,
{
    type PCDAccumulator = DualDLogAccumulator<'a, G1, G2, FS>;
    type PCDVerifierKey = DualPCDVerifierKey<'a, G1, G2, FS>;

    fn succinct_verify(
        &self,
        vk: &Self::PCDVerifierKey,
    ) -> Result<<Self::PCDAccumulator as Accumulator>::Item, PCDError> {
        match self {
            Self::SimpleMarlin(simple_marlin) => {
                // Works because a FinalDarlinVk is a MarlinVk
                let simple_marlin_vk =
                    SimpleMarlinPCDVerifierKey(vk.final_darlin_vk, vk.dlog_vks.0);
                let acc = simple_marlin.succinct_verify(&simple_marlin_vk)?;
                Ok(DualDLogItem {
                    native: vec![acc],
                    non_native: vec![],
                })
            }
            Self::FinalDarlin(final_darlin) => final_darlin.succinct_verify(vk),
        }
    }
}
