//! The proof data struct (and its components) of a final Darlin, i.e. last node of
//! our conversion/exiting chain.
use crate::darlin::{accumulators::dlog::DLogItem, pcd::simple_marlin::MarlinProof};
use algebra::{
    serialize::*, Curve, Group, PrimeField, SemanticallyValid, ToBits, ToConstraintField,
};
use digest::Digest;
use poly_commit::ipa_pc::CommitterKey as DLogCommitterKey;
use rand::RngCore;

/// The `FinalDarlinDeferredData`, assuming that the final node is in G1.
/// This node serves an ordinary Marlin proof plus the dlog accumulators
/// passed from the previous two nodes of the conversion chain.
/// Note: Later the struct could include more elements as we might want to defer
/// addional algebraic checks over G1::BaseField.
#[derive(Default, Clone, Debug, Eq, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct FinalDarlinDeferredData<G1: Curve, G2: Curve> {
    // the dlog accumulator from the previous node, a Rainbow-Marlin node in G2
    pub(crate) previous_acc: DLogItem<G2>,
    // the dlog accumulator from the pre-previous node, a Rainbow-Marlin node in G1
    pub(crate) pre_previous_acc: DLogItem<G1>,
}

impl<G1: Curve, G2: Curve> SemanticallyValid for FinalDarlinDeferredData<G1, G2> {
    fn is_valid(&self) -> bool {
        self.previous_acc.is_valid() && self.pre_previous_acc.is_valid()
    }
}

impl<G1, G2> FinalDarlinDeferredData<G1, G2>
where
    G1: Curve<BaseField = <G2 as Group>::ScalarField>
        + ToConstraintField<<G2 as Group>::ScalarField>,
    G2: Curve<BaseField = <G1 as Group>::ScalarField>
        + ToConstraintField<<G1 as Group>::ScalarField>,
{
    // generates random FinalDarlinDeferredData, for test purposes only.
    pub fn generate_random<R: RngCore, D: Digest>(
        rng: &mut R,
        committer_key_g1: &DLogCommitterKey<G1>,
        committer_key_g2: &DLogCommitterKey<G2>,
    ) -> Self {
        Self {
            previous_acc: DLogItem::generate_random::<_, D>(rng, committer_key_g2),
            pre_previous_acc: DLogItem::generate_random::<_, D>(rng, committer_key_g1),
        }
    }
}

impl<G1, G2> ToConstraintField<G1::ScalarField> for FinalDarlinDeferredData<G1, G2>
where
    G1: Curve<BaseField = <G2 as Group>::ScalarField>
        + ToConstraintField<<G2 as Group>::ScalarField>,
    G2: Curve<BaseField = <G1 as Group>::ScalarField>
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
        for c in g_final_g2.into_iter() {
            fes.append(&mut c.to_field_elements()?);
        }

        // Convert xi_s, which are 128 bit elements from G2::ScalarField, to the native field.
        // We packing the full bit vector into native field elements as efficient as possible (yet
        // still secure).
        let mut xi_s_bits = Vec::new();
        for fe in self.previous_acc.xi_s.0.clone().into_iter() {
            let bits = fe.write_bits();
            // write_bits() outputs a Big Endian bit order representation of fe and the same
            // expects [bool].to_field_elements(): therefore we need to take the last 128 bits,
            // e.g. we need to skip the first MODULUS_BITS - 128 bits.
            debug_assert!(
                <[bool] as ToConstraintField<G2::ScalarField>>::to_field_elements(&bits[to_skip..])
                    .unwrap()[0]
                    == fe
            );
            xi_s_bits.extend_from_slice(&bits[to_skip..]);
        }
        fes.append(&mut xi_s_bits.to_field_elements()?);

        // Convert the pre-previous acc into native field elements.

        // The G_final of the pre-previous node is in G1, hence over G2::ScalarField.
        // We serialize them all to bits and pack them safely into native field elements
        let g_final_g1 = self.pre_previous_acc.g_final.clone();
        let mut g_final_g1_bits = Vec::new();
        for c in g_final_g1.iter() {
            let c_fes = c.to_field_elements()?;
            for fe in c_fes {
                g_final_g1_bits.append(&mut fe.write_bits());
            }
        }
        fes.append(&mut g_final_g1_bits.to_field_elements()?);

        // Although the xi's of the pre-previous node are by default 128 bit elements from G1::ScalarField
        // (we do field arithmetics with them lateron) we do not want waste space.
        // As for the xi's of the previous node, we serialize them all to bits and pack them into native
        // field elements as efficient as possible (yet secure).
        let mut xi_s_bits = Vec::new();
        for fe in self.pre_previous_acc.xi_s.0.clone().into_iter() {
            let bits = fe.write_bits();
            // write_bits() outputs a Big Endian bit order representation of fe and the same
            // expects [bool].to_field_elements(): therefore we need to take the last 128 bits,
            // e.g. we need to skip the first MODULUS_BITS - 128 bits.
            debug_assert!(
                <[bool] as ToConstraintField<G1::ScalarField>>::to_field_elements(&bits[to_skip..])
                    .unwrap()[0]
                    == fe
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
pub struct FinalDarlinProof<G1: Curve, G2: Curve, D: Digest + 'static> {
    /// Full Marlin proof without deferred arithmetics in G1.
    pub proof: MarlinProof<G1, D>,
    /// Deferred accumulators
    pub deferred: FinalDarlinDeferredData<G1, G2>,
}

impl<G1: Curve, G2: Curve, D: Digest> SemanticallyValid for FinalDarlinProof<G1, G2, D> {
    fn is_valid(&self) -> bool {
        self.proof.is_valid() && self.deferred.is_valid()
    }
}
