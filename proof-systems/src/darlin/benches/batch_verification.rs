use algebra::{Group, EndoMulCurve, ToConstraintField};
use blake2::{Blake2s, Digest};
use criterion::*;
use poly_commit::chacha20::FiatShamirChaChaRng;
use poly_commit::{ipa_pc::InnerProductArgPC, fiat_shamir::FiatShamirRng, error::Error as PCError, PolynomialCommitment};
use proof_systems::darlin::pcd::GeneralPCD;
use proof_systems::darlin::{
    proof_aggregator::batch_verify_proofs,
    tests::{final_darlin::generate_test_data as generate_final_darlin_test_data, get_keys},
};
use rand::{thread_rng, SeedableRng};
use rand_xorshift::XorShiftRng;

fn bench_batch_verification<G1: EndoMulCurve, G2: EndoMulCurve, D: Digest, FS: FiatShamirRng<Error = PCError> + 'static>(
    c: &mut Criterion,
    bench_name: &str,
    segment_size: usize,
    max_proofs: Vec<usize>,
) where
    G1: EndoMulCurve<BaseField = <G2 as Group>::ScalarField>
        + ToConstraintField<<G2 as Group>::ScalarField>,
    G2: EndoMulCurve<BaseField = <G1 as Group>::ScalarField>
        + ToConstraintField<<G1 as Group>::ScalarField>,
{
    let rng = &mut XorShiftRng::seed_from_u64(1234567890u64);
    let mut group = c.benchmark_group(bench_name);
    let num_constraints = 1 << 19;

    //Generate DLOG keys
    let params_g1 = InnerProductArgPC::<G1, FS>::setup::<D>(segment_size - 1).unwrap();
    let params_g2 = InnerProductArgPC::<G2, FS>::setup::<D>(segment_size - 1).unwrap();

    let (_, verifier_key_g1, _, verifier_key_g2) = get_keys::<_, _, D>(&params_g1, &params_g2);

    let (final_darlin_pcd, index_vk) = generate_final_darlin_test_data::<D, G1, G2, FS, _>(
        num_constraints - 1,
        segment_size,
        &params_g1,
        &params_g2,
        1,
        rng,
    );

    // Generate proofs and bench
    for num_proofs in max_proofs.into_iter() {
        // Collect PCDs and vks
        let pcds = vec![GeneralPCD::FinalDarlin(final_darlin_pcd[0].clone()); num_proofs];
        let vks = vec![index_vk[0].clone(); num_proofs];

        group.bench_with_input(
            BenchmarkId::from_parameter(num_proofs),
            &num_proofs,
            |bn, _num_proofs| {
                bn.iter(|| {
                    assert!(batch_verify_proofs::<G1, G2, FS, _>(
                        pcds.as_slice(),
                        vks.as_slice(),
                        &verifier_key_g1,
                        &verifier_key_g2,
                        &mut thread_rng()
                    )
                    .unwrap());
                });
            },
        );
    }
    group.finish();
}

// the maximum degree we expect to handle is 2^19, maybe even below, e.g. 2^18
// Segment size |H| => 42, segment size |H|/2 => 84

fn bench_batch_verification_tweedle(c: &mut Criterion) {
    use algebra::curves::tweedle::{dee::DeeJacobian as TweedleDee, dum::DumJacobian as TweedleDum};

    bench_batch_verification::<TweedleDee, TweedleDum, Blake2s, FiatShamirChaChaRng<Blake2s>>(
        c,
        "tweedle-dee, |H| = segment_size = 1 << 19, proofs",
        1 << 19,
        vec![10, 50, 100, 200],
    );

    bench_batch_verification::<TweedleDee, TweedleDum, Blake2s, FiatShamirChaChaRng<Blake2s>>(
        c,
        "tweedle-dee, |H| = 1 << 19, segment_size = |H|/2, proofs",
        1 << 18,
        vec![10, 50, 100, 200],
    );

    bench_batch_verification::<TweedleDee, TweedleDum, Blake2s, FiatShamirChaChaRng<Blake2s>>(
        c,
        "tweedle-dee, |H| = 1 << 19, segment_size = |H|/4, proofs",
        1 << 17,
        vec![10, 50, 100, 200],
    );
}

criterion_group!(
name = batch_verification;
config = Criterion::default().sample_size(10);
targets = bench_batch_verification_tweedle
);

criterion_main!(batch_verification);
