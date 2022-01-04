//! The data structures for Coboundary Marlin:
//!     - prover and verifier key,
//!     - the SNARK proof,
//! and implementations for serialization and deserialization.
use crate::iop::indexer::*;
use crate::{Vec, IOP};
use algebra::serialize::*;
use algebra::{Curve, Group, ToBytes};
use derivative::Derivative;
use poly_commit::PolynomialCommitment;

/// The universal public parameters for the argument system.
pub type UniversalSRS<G, PC> = <PC as PolynomialCommitment<G>>::Parameters;

/// The verification key for a specific R1CS.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Debug(bound = ""),
    Eq(bound = ""),
    PartialEq(bound = "")
)]
#[derive(CanonicalSerialize, CanonicalDeserialize)]
pub struct VerifierKey<G: Curve> {
    /// Stores R1CS metrics as usually supplied by the constraint system.
    pub index_info: IndexInfo<G::ScalarField>,
}

/// The prover key for a specific R1CS.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Debug(bound = ""),
    Eq(bound = ""),
    PartialEq(bound = "")
)]
#[derive(CanonicalSerialize, CanonicalDeserialize)]
pub struct ProverKey<G: Curve> {
    /// The index verifier key.
    pub index_vk: VerifierKey<G>,
    /// The index itself.
    pub index: Index<G::ScalarField>,
}

/// The SNARK proof itself.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Debug(bound = ""),
    Eq(bound = ""),
    PartialEq(bound = "")
)]
pub struct Proof<G: Curve, PC: PolynomialCommitment<G>> {
    /// Commitments to the polynomials produced by the prover
    pub commitments: Vec<Vec<PC::Commitment>>,
    /// Evaluations of these polynomials.
    pub evaluations: Vec<G::ScalarField>,
    /// A batch evaluation proof from the polynomial commitment.
    pub pc_proof: PC::MultiPointProof,
}

impl<G: Curve, PC: PolynomialCommitment<G>> Proof<G, PC> {
    /// Construct a new proof.
    pub fn new(
        commitments: Vec<Vec<PC::Commitment>>,
        evaluations: Vec<G::ScalarField>,
        pc_proof: PC::MultiPointProof,
    ) -> Self {
        Self {
            commitments,
            evaluations,
            pc_proof,
        }
    }

    /// Prints information about the size of the proof.
    pub fn print_size_info(&self) {
        let size_of_fe_in_bytes = G::ScalarField::zero().serialized_size();
        let mut num_comms_without_degree_bounds = 0;
        let mut size_bytes_comms_without_degree_bounds = 0;
        let mut size_bytes_proofs = 0;
        for c in self.commitments.iter().flatten() {
            num_comms_without_degree_bounds += 1;
            size_bytes_comms_without_degree_bounds += c.serialized_size();
        }

        size_bytes_proofs += self.pc_proof.serialized_size();

        let num_evals = self.evaluations.len();
        let evals_size_in_bytes = num_evals * size_of_fe_in_bytes;
        let arg_size =
            size_bytes_comms_without_degree_bounds + size_bytes_proofs + evals_size_in_bytes;
        let stats = format!(
            "Argument size in bytes: {}\n\n\
             Number of commitments without degree bounds: {}\n\
             Size (in bytes) of commitments without degree bounds: {}\n\
             Size (in bytes) of evaluation proofs: {}\n\n\
             Number of evaluations: {}\n\
             Size (in bytes) of evaluations: {}\n",
            arg_size,
            num_comms_without_degree_bounds,
            size_bytes_comms_without_degree_bounds,
            size_bytes_proofs,
            num_evals,
            evals_size_in_bytes,
        );
        add_to_trace!(|| "Statistics about proof", || stats);
    }
}

/*
    Implement SemanticallyValid for VerifierKey, ProverKey, and Proof.
*/

impl<G: Curve> algebra::SemanticallyValid for VerifierKey<G> {
    fn is_valid(&self) -> bool {
        true
    }
}

impl<G: Curve> algebra::SemanticallyValid for ProverKey<G> {
    fn is_valid(&self) -> bool {
        self.index_vk.is_valid() && self.index.is_valid()
    }
}

impl<G: Curve, PC: PolynomialCommitment<G>> algebra::SemanticallyValid for Proof<G, PC> {
    fn is_valid(&self) -> bool {
        // Check commitments number and validity
        let num_rounds = 2;
        let comms_per_round = vec![3, 3];

        // Check commitments are grouped into correct num_rounds
        if self.commitments.len() != num_rounds {
            return false;
        };

        // Check that each round has the expected number of commitments
        for i in 0..comms_per_round.len() {
            if self.commitments[i].len() != comms_per_round[i] {
                return false;
            };
        }

        // Check evaluations num
        let evaluations_num = IOP::<G::ScalarField>::PROVER_POLYNOMIALS.len() +
            1 // boundary polynomial is evaluated at two different points
        ;

        self.commitments.is_valid() &&  // Check that each commitment is valid
            self.evaluations.len() == evaluations_num && // Check correct number of evaluations
            self.evaluations.is_valid() && // Check validity of each evaluation
            self.pc_proof.is_valid() // Check validity of the batch proof.
    }
}

/*
    Serialization and Deserialization utilities.
*/

impl<G: Curve> ToBytes for VerifierKey<G> {
    #[inline]
    fn write<W: Write>(&self, writer: W) -> std::io::Result<()> {
        self.serialize_without_metadata(writer)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", e)))
    }
}

impl<G: Curve, PC: PolynomialCommitment<G>> CanonicalSerialize for Proof<G, PC> {
    fn serialize<W: Write>(&self, mut writer: W) -> Result<(), SerializationError> {
        // Serialize commitments: we know in advance exactly how many polynomials will be
        // committed, so we can skip writing the corresponding sizes.
        for comm in self.commitments.iter().flatten() {
            CanonicalSerialize::serialize(comm, &mut writer)?;
        }

        // Serialize evaluations: again, we know the number in advance.
        for eval in self.evaluations.iter() {
            CanonicalSerialize::serialize(eval, &mut writer)?;
        }

        // Serialize pc_proof
        CanonicalSerialize::serialize(&self.pc_proof, &mut writer)
    }

    fn serialized_size(&self) -> usize {
        let mut size = 0;

        // Commitments size
        self.commitments
            .iter()
            .flatten()
            .for_each(|comm| size += comm.serialized_size());

        let evaluations_num = IOP::<G::ScalarField>::PROVER_POLYNOMIALS.len() +
            1 // boundary polynomial is evaluated at two different points
        ;

        // Evaluations size
        size += evaluations_num * self.evaluations[0].serialized_size();

        // PC proof size
        size += self.pc_proof.serialized_size();

        size
    }

    fn serialize_without_metadata<W: Write>(
        &self,
        mut writer: W,
    ) -> Result<(), SerializationError> {
        // Serialize commitments: we know in advance exactly how many polynomials will be
        // committed, so we can skip writing the corresponding sizes.
        for comm in self.commitments.iter().flatten() {
            CanonicalSerialize::serialize_without_metadata(comm, &mut writer)?;
        }

        // Serialize evaluations: again, we know the number in advance.
        for eval in self.evaluations.iter() {
            CanonicalSerialize::serialize_without_metadata(eval, &mut writer)?;
        }

        // Serialize pc_proof
        CanonicalSerialize::serialize_without_metadata(&self.pc_proof, &mut writer)
    }

    #[inline]
    fn serialize_uncompressed<W: Write>(&self, mut writer: W) -> Result<(), SerializationError> {
        // Serialize commitments: we know in advance exactly how many polynomials will be
        // committed, so we can skip writing the corresponding sizes.
        for comm in self.commitments.iter().flatten() {
            CanonicalSerialize::serialize_uncompressed(comm, &mut writer)?;
        }

        // Serialize evaluations: again, we know the number in advance.
        for eval in self.evaluations.iter() {
            CanonicalSerialize::serialize_uncompressed(eval, &mut writer)?;
        }

        // Serialize pc_proof
        CanonicalSerialize::serialize_uncompressed(&self.pc_proof, &mut writer)
    }

    #[inline]
    fn uncompressed_size(&self) -> usize {
        let mut size = 0;

        // Commitments size
        self.commitments
            .iter()
            .flatten()
            .for_each(|comm| size += comm.uncompressed_size());

        let evaluations_num = IOP::<G::ScalarField>::PROVER_POLYNOMIALS.len() +
            1 // boundary polynomial is evaluated at two different points
            ;

        // Evaluations size
        size += evaluations_num * self.evaluations[0].uncompressed_size();

        // PC proof size
        size += self.pc_proof.uncompressed_size();

        size
    }
}

impl<G: Curve, PC: PolynomialCommitment<G>> CanonicalDeserialize for Proof<G, PC> {
    fn deserialize<R: Read>(mut reader: R) -> Result<Self, SerializationError> {
        // Deserialize commitments
        let num_rounds = 2;
        let comms_per_round = vec![3, 3];
        let mut commitments = Vec::with_capacity(num_rounds);

        for i in 0..num_rounds {
            // Deserialize round commitments
            let mut round_comms = Vec::with_capacity(comms_per_round[i]);

            for _ in 0..comms_per_round[i] {
                let comm: PC::Commitment = CanonicalDeserialize::deserialize(&mut reader)?;
                round_comms.push(comm);
            }

            commitments.push(round_comms);
        }

        // Deserialize evaluations
        let evaluations_num = IOP::<G::ScalarField>::PROVER_POLYNOMIALS.len() +
            1 // boundary polynomial is evaluated at two different points
        ;

        let mut evaluations = Vec::with_capacity(evaluations_num);
        for _ in 0..evaluations_num {
            let eval: G::ScalarField = CanonicalDeserialize::deserialize(&mut reader)?;
            evaluations.push(eval);
        }

        // Deserialize pc_proof
        let pc_proof = CanonicalDeserialize::deserialize(&mut reader)?;

        Ok(Self {
            commitments,
            evaluations,
            pc_proof,
        })
    }

    fn deserialize_unchecked<R: Read>(mut reader: R) -> Result<Self, SerializationError> {
        // Deserialize commitments
        let num_rounds = 2;
        let comms_per_round = vec![3, 3];
        let mut commitments = Vec::with_capacity(num_rounds);

        for i in 0..num_rounds {
            // Deserialize round commitments
            let mut round_comms = Vec::with_capacity(comms_per_round[i]);

            for _ in 0..comms_per_round[i] {
                let comm: PC::Commitment =
                    CanonicalDeserialize::deserialize_unchecked(&mut reader)?;
                round_comms.push(comm);
            }

            commitments.push(round_comms);
        }

        // Deserialize evaluations
        let evaluations_num = IOP::<G::ScalarField>::PROVER_POLYNOMIALS.len() +
            1 // boundary polynomial is evaluated at two different points
            ;

        let mut evaluations = Vec::with_capacity(evaluations_num);
        for _ in 0..evaluations_num {
            let eval: G::ScalarField = CanonicalDeserialize::deserialize_unchecked(&mut reader)?;
            evaluations.push(eval);
        }

        // Deserialize pc_proof
        let pc_proof = CanonicalDeserialize::deserialize_unchecked(&mut reader)?;

        Ok(Self {
            commitments,
            evaluations,
            pc_proof,
        })
    }

    #[inline]
    fn deserialize_uncompressed<R: Read>(mut reader: R) -> Result<Self, SerializationError> {
        // Deserialize commitments
        let num_rounds = 2;
        let comms_per_round = vec![3, 3];
        let mut commitments = Vec::with_capacity(num_rounds);

        for i in 0..num_rounds {
            // Deserialize round commitments
            let mut round_comms = Vec::with_capacity(comms_per_round[i]);

            for _ in 0..comms_per_round[i] {
                let comm: PC::Commitment =
                    CanonicalDeserialize::deserialize_uncompressed(&mut reader)?;
                round_comms.push(comm);
            }

            commitments.push(round_comms);
        }

        // Deserialize evaluations
        let evaluations_num = IOP::<G::ScalarField>::PROVER_POLYNOMIALS.len() +
            1 // boundary polynomial is evaluated at two different points
            ;

        let mut evaluations = Vec::with_capacity(evaluations_num);
        for _ in 0..evaluations_num {
            let eval: G::ScalarField = CanonicalDeserialize::deserialize_uncompressed(&mut reader)?;
            evaluations.push(eval);
        }

        // Deserialize pc_proof
        let pc_proof = CanonicalDeserialize::deserialize_uncompressed(&mut reader)?;

        Ok(Self {
            commitments,
            evaluations,
            pc_proof,
        })
    }

    #[inline]
    fn deserialize_uncompressed_unchecked<R: Read>(
        mut reader: R,
    ) -> Result<Self, SerializationError> {
        // Deserialize commitments
        let num_rounds = 2;
        let comms_per_round = vec![3, 3];
        let mut commitments = Vec::with_capacity(num_rounds);

        for i in 0..num_rounds {
            // Deserialize round commitments
            let mut round_comms = Vec::with_capacity(comms_per_round[i]);

            for _ in 0..comms_per_round[i] {
                let comm: PC::Commitment =
                    CanonicalDeserialize::deserialize_uncompressed_unchecked(&mut reader)?;
                round_comms.push(comm);
            }

            commitments.push(round_comms);
        }

        // Deserialize evaluations
        let evaluations_num = IOP::<G::ScalarField>::PROVER_POLYNOMIALS.len() +
            1 // boundary polynomial is evaluated at two different points
            ;

        let mut evaluations = Vec::with_capacity(evaluations_num);
        for _ in 0..evaluations_num {
            let eval: G::ScalarField =
                CanonicalDeserialize::deserialize_uncompressed_unchecked(&mut reader)?;
            evaluations.push(eval);
        }

        // Deserialize pc_proof
        let pc_proof = CanonicalDeserialize::deserialize_uncompressed_unchecked(&mut reader)?;

        Ok(Self {
            commitments,
            evaluations,
            pc_proof,
        })
    }
}
