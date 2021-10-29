use {
    algebra::curves::tweedle::{
        dum::{
            Affine as DumAffine, Projective as DumProjective, TweedledumParameters,
        },
        dee::{
            Affine as DeeAffine, Projective as DeeProjective,
        },
    },
    primitives::crh::poseidon::parameters::tweedle_dee::{
        TweedleFrBatchPoseidonHash, TweedleFrPoseidonHash,
    },
    primitives::merkle_tree::tweedle_dee::TWEEDLE_DEE_MHT_POSEIDON_PARAMETERS,
};

generate_all_types_and_functions!(
    DumAffine,
    DumProjective,
    TweedledumParameters,
    TweedleFrPoseidonHash,
    TweedleFrBatchPoseidonHash,
    TWEEDLE_DEE_MHT_POSEIDON_PARAMETERS,
    2
);

#[cfg(feature = "darlin")]
generate_darlin_types!(DeeAffine, DeeProjective);
