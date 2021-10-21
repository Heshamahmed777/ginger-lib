use algebra::Field;
use r1cs_core::{ConstraintSystem, SynthesisError};
use r1cs_std::prelude::*;
use primitives::signature::{
    SignatureScheme, FieldBasedSignatureScheme,
};

pub mod schnorr;

pub trait SigRandomizePkGadget<S: SignatureScheme, ConstraintF: Field> {
    type ParametersGadget: AllocGadget<S::Parameters, ConstraintF> + Clone;

    type PublicKeyGadget: ToBytesGadget<ConstraintF>
        + EqGadget<ConstraintF>
        + AllocGadget<S::PublicKey, ConstraintF>
        + Clone;

    fn check_randomization_gadget<CS: ConstraintSystem<ConstraintF>>(
        cs: CS,
        parameters: &Self::ParametersGadget,
        public_key: &Self::PublicKeyGadget,
        randomness: &[UInt8],
    ) -> Result<Self::PublicKeyGadget, SynthesisError>;
}


pub trait FieldBasedSigGadget<S: FieldBasedSignatureScheme, ConstraintF: Field> {

    type DataGadget:      FieldGadget<ConstraintF, ConstraintF>;
    type SignatureGadget: AllocGadget<S::Signature, ConstraintF>;
    type PublicKeyGadget: AllocGadget<S::PublicKey, ConstraintF>;

    /// Enforce `signature` verification with `public_key` on `message`, returning a Boolean
    /// enforced to be `true` if signature verification is successful, and `false` otherwise.
    fn enforce_signature_verdict<CS: ConstraintSystem<ConstraintF>>(
        cs: CS,
        public_key: &Self::PublicKeyGadget,
        signature:  &Self::SignatureGadget,
        message:    &[Self::DataGadget],
    ) -> Result<Boolean, SynthesisError>;

    ///Enforce `signature` verification with `public_key` on `message` to be successful.
    fn enforce_signature_verification<CS: ConstraintSystem<ConstraintF>>(
        cs: CS,
        public_key: &Self::PublicKeyGadget,
        signature:  &Self::SignatureGadget,
        message:    &[Self::DataGadget],
    ) -> Result<(), SynthesisError>;
}