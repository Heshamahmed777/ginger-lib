//! The proof data struct (and its components) of a final Darlin, i.e. last node of
//! our conversion/exiting chain.
use crate::darlin::{accumulators::dlog::DLogItem, pcd::simple_marlin::MarlinProof};
use algebra::{
    serialize::*, EndoMulCurve, Group, PrimeField, SemanticallyValid, ToBits, ToConstraintField,
    UniformRand,
};
use fiat_shamir::FiatShamirRng;
use poly_commit::ipa_pc::{
    CommitterKey as DLogCommitterKey, InnerProductArgPC, SuccinctCheckPolynomial,
};
use rand::RngCore;

/// The `FinalDarlinDeferredData`, assuming that the final node is in G1.
/// This node serves an ordinary Marlin proof plus the dlog accumulators
/// passed from the previous two nodes of the conversion chain.
/// Note: Later the struct could include more elements as we might want to defer
/// addional algebraic checks over G1::BaseField.
#[derive(Default, Clone, Debug, Eq, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct FinalDarlinDeferredData<G1: EndoMulCurve, G2: EndoMulCurve> {
    // the dlog accumulator from the previous node, a Rainbow-Marlin node in G2
    pub(crate) previous_acc: DLogItem<G2>,
    // the dlog accumulator from the pre-previous node, a Rainbow-Marlin node in G1
    pub(crate) pre_previous_acc: DLogItem<G1>,
}

impl<G1: EndoMulCurve, G2: EndoMulCurve> SemanticallyValid for FinalDarlinDeferredData<G1, G2> {
    fn is_valid(&self) -> bool {
        self.previous_acc.is_valid() && self.pre_previous_acc.is_valid()
    }
}

impl<G1, G2> FinalDarlinDeferredData<G1, G2>
where
    G1: EndoMulCurve<BaseField = <G2 as Group>::ScalarField>
        + ToConstraintField<<G2 as Group>::ScalarField>,
    G2: EndoMulCurve<BaseField = <G1 as Group>::ScalarField>
        + ToConstraintField<<G1 as Group>::ScalarField>,
{
    // generates random FinalDarlinDeferredData, for test purposes only.
    pub fn generate_random<R: RngCore, FS: FiatShamirRng>(
        rng: &mut R,
        committer_key_g1: &DLogCommitterKey<G1>,
        committer_key_g2: &DLogCommitterKey<G2>,
    ) -> Self {
        // Generate valid accumulator over G1 starting from random xi_s
        let log_key_len_g1 = algebra::log2(committer_key_g1.comm_key.len());
        let random_xi_s_g1 = SuccinctCheckPolynomial::<G1>::from_chals(
            (0..log_key_len_g1 as usize)
                .map(|_| u128::rand(rng).into())
                .collect(),
        );
        let g_final_g1 = InnerProductArgPC::<G1, FS>::inner_commit(
            committer_key_g1.comm_key.as_slice(),
            random_xi_s_g1.compute_coeffs().as_slice(),
            None,
            None,
        )
        .unwrap();

        let acc_g1 = DLogItem::<G1> {
            g_final: g_final_g1,
            xi_s: random_xi_s_g1,
        };

        // Generate valid accumulator over G2 starting from random xi_s
        let log_key_len_g2 = algebra::log2(committer_key_g2.comm_key.len());
        let random_xi_s_g2 = SuccinctCheckPolynomial::<G2>::from_chals(
            (0..log_key_len_g2 as usize)
                .map(|_| u128::rand(rng).into())
                .collect(),
        );

        let g_final_g2 = InnerProductArgPC::<G2, FS>::inner_commit(
            committer_key_g2.comm_key.as_slice(),
            random_xi_s_g2.compute_coeffs().as_slice(),
            None,
            None,
        )
        .unwrap();

        let acc_g2 = DLogItem::<G2> {
            g_final: g_final_g2,
            xi_s: random_xi_s_g2,
        };

        // Return accumulators in deferred struct
        Self {
            previous_acc: acc_g2,
            pre_previous_acc: acc_g1,
        }
    }
}

impl<G1, G2> ToConstraintField<G1::ScalarField> for FinalDarlinDeferredData<G1, G2>
where
    G1: EndoMulCurve<BaseField = <G2 as Group>::ScalarField>
        + ToConstraintField<<G2 as Group>::ScalarField>,
    G2: EndoMulCurve<BaseField = <G1 as Group>::ScalarField>
        + ToConstraintField<<G1 as Group>::ScalarField>,
{
    /// Conversion of the MarlinDeferredData to circuit inputs, which are elements
    /// over G1::ScalarField.
    fn to_field_elements(&self) -> Result<Vec<G1::ScalarField>, Box<dyn std::error::Error>> {
        let to_skip = <G1::ScalarField as PrimeField>::size_in_bits() - 128;
        let mut fes = Vec::new();

        // Convert previous_acc into G1::ScalarField field elements (the circuit field,
        // called native in the sequel)

        // The G_final of the previous node consists of native field elements only
        let g_final_g2 = self.previous_acc.g_final.clone();
        fes.append(&mut g_final_g2.to_field_elements()?);

        // Convert xi_s, which are 128 bit elements from G2::ScalarField, to the native field.
        // We packing the full bit vector into native field elements as efficient as possible (yet
        // still secure).
        let mut xi_s_bits = Vec::new();
        for fe in self.previous_acc.xi_s.chals.iter() {
            let bits = fe.write_bits();
            // write_bits() outputs a Big Endian bit order representation of fe and the same
            // expects [bool].to_field_elements(): therefore we need to take the last 128 bits,
            // e.g. we need to skip the first MODULUS_BITS - 128 bits.
            debug_assert!(
                <[bool] as ToConstraintField<G2::ScalarField>>::to_field_elements(&bits[to_skip..])
                    .unwrap()[0]
                    == *fe
            );
            xi_s_bits.extend_from_slice(&bits[to_skip..]);
        }
        fes.append(&mut xi_s_bits.to_field_elements()?);

        // Convert the pre-previous acc into native field elements.

        // The G_final of the pre-previous node is in G1, hence over G2::ScalarField.
        // We serialize them all to bits and pack them safely into native field elements
        let g_final_g1 = self.pre_previous_acc.g_final.clone();
        let mut g_final_g1_bits = Vec::new();
        let c_fes = g_final_g1.to_field_elements()?;
        for fe in c_fes {
            g_final_g1_bits.append(&mut fe.write_bits());
        }
        fes.append(&mut g_final_g1_bits.to_field_elements()?);

        // Although the xi's of the pre-previous node are by default 128 bit elements from G1::ScalarField
        // (we do field arithmetics with them lateron) we do not want waste space.
        // As for the xi's of the previous node, we serialize them all to bits and pack them into native
        // field elements as efficient as possible (yet secure).
        let mut xi_s_bits = Vec::new();
        for fe in self.pre_previous_acc.xi_s.chals.iter() {
            let bits = fe.write_bits();
            // write_bits() outputs a Big Endian bit order representation of fe and the same
            // expects [bool].to_field_elements(): therefore we need to take the last 128 bits,
            // e.g. we need to skip the first MODULUS_BITS - 128 bits.
            debug_assert!(
                <[bool] as ToConstraintField<G1::ScalarField>>::to_field_elements(&bits[to_skip..])
                    .unwrap()[0]
                    == *fe
            );
            xi_s_bits.extend_from_slice(&bits[to_skip..]);
        }
        fes.append(&mut xi_s_bits.to_field_elements()?);

        Ok(fes)
    }
}

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Debug(bound = ""),
    Eq(bound = ""),
    PartialEq(bound = "")
)]
#[derive(CanonicalSerialize, CanonicalDeserialize)]
/// A FinalDarlinProof has two dlog accumulators, one from the previous, and on from the
/// pre-previous node of the conversion chain.
pub struct FinalDarlinProof<G1: EndoMulCurve, G2: EndoMulCurve, FS: FiatShamirRng + 'static> {
    /// Full Marlin proof without deferred arithmetics in G1.
    pub proof: MarlinProof<G1, FS>,
    /// Deferred accumulators
    pub deferred: FinalDarlinDeferredData<G1, G2>,
}

impl<G1: EndoMulCurve, G2: EndoMulCurve, FS: FiatShamirRng + 'static> SemanticallyValid
    for FinalDarlinProof<G1, G2, FS>
{
    fn is_valid(&self) -> bool {
        self.proof.is_valid() && self.deferred.is_valid()
    }
}
