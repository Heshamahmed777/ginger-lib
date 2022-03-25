use crate::{
    DomainExtendedPolyCommitVerifierGadget, DomainExtendedPolynomialCommitment, Error as PolyError,
    Evaluations, LabeledCommitment, LabeledCommitmentGadget, LabeledPolynomial, PCKey, Polynomial,
    PolynomialCommitment, PolynomialCommitmentVerifierGadget, PolynomialLabel, QueryMap,
};
use algebra::{Field, Group, PrimeField, SemanticallyValid, ToBits, UniformRand};
use blake2::Blake2s;
use fiat_shamir::constraints::FiatShamirRngGadget;
use fiat_shamir::FiatShamirRng;
use r1cs_core::{
    ConstraintSystem, ConstraintSystemAbstract, ConstraintSystemDebugger, SynthesisError,
    SynthesisMode,
};
use r1cs_std::alloc::AllocGadget;
use r1cs_std::boolean::Boolean;
use r1cs_std::fields::nonnative::nonnative_field_gadget::NonNativeFieldGadget;
use r1cs_std::groups::GroupGadget;
use rand::{thread_rng, Rng};
use rand_core::RngCore;
use std::cmp;
use std::collections::BTreeSet;

#[derive(Copy, Clone, PartialEq, Debug)]
enum NegativeTestType {
    Values,
    Commitments,
}

#[derive(Copy, Clone, Default)]
struct TestInfo {
    /// number of random instances to be tested
    num_iters: usize,
    /// The degree bound for sampling the supported degree of
    /// the non-extended scheme.
    max_degree: Option<usize>,
    /// The optional maximum degree supported by the non-extended scheme
    /// (i.e. the "segment size")
    supported_degree: Option<usize>,
    /// The number of polynomials
    num_polynomials: usize,
    /// the number of query points
    max_num_queries: usize,
    /// if the polynomials must be segmented or not
    segmented: bool,
    /// negative type for the test
    negative_type: Option<NegativeTestType>,
}

fn value_for_alloc<F: Field, R: RngCore>(
    value: &F,
    negative_type: &Option<NegativeTestType>,
    rng: &mut R,
) -> F {
    if let Some(NegativeTestType::Values) = negative_type {
        F::rand(rng)
    } else {
        value.clone()
    }
}

fn commitment_for_alloc<G: Group, PC: PolynomialCommitment<G>>(
    commitment: &PC::Commitment,
    negative_type: &Option<NegativeTestType>,
) -> PC::Commitment {
    if let Some(NegativeTestType::Commitments) = negative_type {
        commitment.clone().double()
    } else {
        commitment.clone()
    }
}

fn setup_pc_parameters<G: Group, PC: PolynomialCommitment<G>, R: RngCore>(
    test_conf: &TestInfo,
    rng: &mut R,
) -> Result<(usize, PC::CommitterKey, PC::VerifierKey), PC::Error> {
    let max_degree: usize = test_conf.max_degree.unwrap_or(rng.gen_range(1..=256));
    let supported_degree: usize = test_conf
        .supported_degree
        .unwrap_or(rng.gen_range(1..=max_degree));
    assert!(supported_degree <= max_degree);

    let pp = PC::setup::<Blake2s>(max_degree)?;
    // Note that we need to ensure that the degree provided as input to trim function must be >= 1,
    // but supported_degree may be 0
    let (ck, vk) = (
        pp.0.trim(cmp::max(supported_degree, 1))?,
        pp.1.trim(cmp::max(supported_degree, 1))?,
    );

    Ok((supported_degree, ck, vk))
}

fn generate_labeled_polynomial<R: RngCore, F: Field>(
    segmented: bool,
    label: PolynomialLabel,
    supported_degree: usize,
    rng: &mut R,
) -> LabeledPolynomial<F> {
    let poly_degree: usize = if segmented {
        rng.gen_range(supported_degree..=10 * supported_degree)
    } else {
        rng.gen_range(0..=supported_degree)
    };
    let polynomial = Polynomial::rand(poly_degree, rng);

    let is_hiding: bool = rng.gen();

    LabeledPolynomial::new(label, polynomial, is_hiding)
}

fn alloc_gadgets_for_succinct_verify<
    ConstraintF: PrimeField,
    G: Group<BaseField = ConstraintF>,
    PC: PolynomialCommitment<G>,
    PCG: PolynomialCommitmentVerifierGadget<ConstraintF, G, PC>,
    CS: ConstraintSystemAbstract<ConstraintF>,
>(
    mut cs: CS,
    vk: &PC::VerifierKey,
    commitments: &[LabeledCommitment<PC::Commitment>],
    fs_seed: Vec<u8>,
    test_type: &Option<NegativeTestType>,
) -> Result<
    (
        PCG::VerifierKeyGadget,
        Vec<LabeledCommitmentGadget<ConstraintF, PC::Commitment, PCG::CommitmentGadget>>,
        PCG::RandomOracleGadget,
    ),
    SynthesisError,
> {
    let vk_gadget = PCG::VerifierKeyGadget::alloc(cs.ns(|| "alloc verifier key"), || Ok(vk))?;
    let mut labeled_comms = Vec::with_capacity(commitments.len());
    for comm in commitments {
        let label = comm.label();
        let comm_gadget = PCG::CommitmentGadget::alloc(
            cs.ns(|| format!("alloc commitment with label {}", label)),
            || Ok(commitment_for_alloc::<G, PC>(comm.commitment(), test_type)),
        )?;
        labeled_comms.push(LabeledCommitmentGadget::new(label.clone(), comm_gadget));
    }
    let fs_gadget = PCG::RandomOracleGadget::init_from_seed(cs.ns(|| "init fs oracle"), fs_seed)?;

    Ok((vk_gadget, labeled_comms, fs_gadget))
}

fn test_succinct_verify_template<
    ConstraintF: PrimeField,
    G: Group<BaseField = ConstraintF>,
    PC: PolynomialCommitment<G>,
    PCG: PolynomialCommitmentVerifierGadget<ConstraintF, G, PC>,
>(
    test_conf: TestInfo,
) -> Result<(), PCG::Error> {
    let rng = &mut thread_rng();
    for _ in 0..test_conf.num_iters {
        let (supported_degree, ck, vk) = setup_pc_parameters::<G, PC, _>(&test_conf, rng)?;

        let labeled_polynomial =
            generate_labeled_polynomial(test_conf.segmented, "".to_string(), supported_degree, rng);
        let (polynomial, is_hiding) = (
            labeled_polynomial.polynomial(),
            labeled_polynomial.is_hiding(),
        );
        let (commitment, randomness) = PC::commit(&ck, polynomial, is_hiding, Some(rng))?;

        let point = G::ScalarField::rand(rng);
        let value = polynomial.evaluate(point);
        let fs_seed = String::from("TEST_SEED").into_bytes();
        let mut fs_rng =
            PC::RandomOracle::from_seed(fs_seed.clone()).map_err(|err| PolyError::from(err))?;
        let proof = PC::open(
            &ck,
            polynomial.clone(),
            point,
            is_hiding,
            randomness,
            &mut fs_rng,
            Some(rng),
        )?;
        // check that verify with primitive succeeds
        let mut fs_rng =
            PC::RandomOracle::from_seed(fs_seed.clone()).map_err(|err| PolyError::from(err))?;
        let v_state = PC::succinct_verify(&vk, &commitment, point, value, &proof, &mut fs_rng)?;
        if v_state.is_none() {
            Err(PolyError::FailedSuccinctCheck)?
        }

        let mut cs = ConstraintSystem::<ConstraintF>::new(SynthesisMode::Debug);

        let (vk_gadget, labeled_commitments, mut fs_gadget) =
            alloc_gadgets_for_succinct_verify::<ConstraintF, G, PC, PCG, _>(
                cs.ns(|| "alloc gadgets for verify"),
                &vk,
                &vec![LabeledCommitment::<PC::Commitment>::new(
                    "".to_string(),
                    commitment.clone(),
                )]
                .as_slice(),
                fs_seed,
                &test_conf.negative_type,
            )?;
        let commitment_gadget = labeled_commitments.get(0).unwrap().commitment();

        let point_gadget = NonNativeFieldGadget::<G::ScalarField, ConstraintF>::alloc(
            cs.ns(|| "alloc evaluation point"),
            || Ok(point),
        )?;
        let value_gadget = Boolean::alloc_input_vec(
            cs.ns(|| "alloc polynomial evalauation on point"),
            value_for_alloc::<G::ScalarField, _>(&value, &test_conf.negative_type, rng)
                .write_bits()
                .as_slice(),
        )?;
        let proof_gadget = PCG::ProofGadget::alloc(cs.ns(|| "alloc opening proof"), || Ok(proof))?;
        let _v_state_gadget = PCG::succinct_verify(
            cs.ns(|| "succinct-verify"),
            &vk_gadget,
            commitment_gadget,
            &point_gadget,
            &value_gadget,
            &proof_gadget,
            &mut fs_gadget,
        )?;
        let successful_test = cs.is_satisfied() ^ test_conf.negative_type.is_some();
        if !successful_test {
            println!("{:?}", cs.which_is_unsatisfied());
        }
        assert!(successful_test);

        // test mul_bits_fixed_base for commitments
        let bits = fs_gadget
            .enforce_get_challenge::<_, 128>(cs.ns(|| "get random bits for mul_bits_fixed_base"))?;
        PCG::CommitmentGadget::mul_bits_fixed_base(&commitment, cs, &bits[..])?;
    }
    Ok(())
}

fn test_multi_point_multi_poly_verify<
    ConstraintF: PrimeField,
    G: Group<BaseField = ConstraintF>,
    PC: PolynomialCommitment<G>,
    PCG: PolynomialCommitmentVerifierGadget<ConstraintF, G, PC>,
>(
    test_conf: TestInfo,
) -> Result<(), PCG::Error> {
    let rng = &mut thread_rng();
    for _ in 0..test_conf.num_iters {
        let mut cs = ConstraintSystem::<ConstraintF>::new(SynthesisMode::Debug);

        let (supported_degree, ck, vk) = setup_pc_parameters::<G, PC, _>(&test_conf, rng)?;

        let max_num_polynomials = test_conf.num_polynomials;
        let num_polynomials = rng.gen_range(1..=max_num_polynomials);
        let max_num_points = test_conf.max_num_queries;
        let num_points = rng.gen_range(1..=max_num_points);

        let mut polynomials = Vec::with_capacity(num_polynomials);
        let mut labels = Vec::with_capacity(num_polynomials);
        for i in 0..num_polynomials {
            let label = format!("Test polynomial {}", i);
            labels.push(label.clone());
            let labeled_polynomial =
                generate_labeled_polynomial(test_conf.segmented, label, supported_degree, rng);

            polynomials.push(labeled_polynomial);
        }

        let (comms, rands) = PC::commit_many(&ck, &polynomials, Some(rng))?;

        assert!(comms.is_valid());
        let mut points = QueryMap::new();
        let mut point_gadgets = QueryMap::new();
        let mut values = Evaluations::new();
        let mut value_gadgets = Evaluations::new();
        for i in 0..num_points {
            let point_label = format!("point label {}", i);
            let point = G::ScalarField::rand(rng);
            let point_gadget = NonNativeFieldGadget::<G::ScalarField, ConstraintF>::alloc(
                cs.ns(|| format!("alloc gadget for point {}", i)),
                || Ok(point),
            )?;
            let mut poly_labels_set = BTreeSet::new();
            for (j, labeled_poly) in polynomials.iter().enumerate() {
                let label = labeled_poly.label();
                let poly = labeled_poly.polynomial();
                let value = poly.evaluate(point);
                let value_gadget = NonNativeFieldGadget::<G::ScalarField, ConstraintF>::alloc(
                    cs.ns(|| {
                        format!(
                            "alloc gadget for evaluation of point  {} over poly {}",
                            i, j
                        )
                    }),
                    || Ok(value_for_alloc(&value, &test_conf.negative_type, rng)),
                )?;
                poly_labels_set.insert(label.clone());
                values.insert((label.clone(), point_label.clone()), value);
                value_gadgets.insert((label.clone(), point_label.clone()), value_gadget);
            }
            points.insert(point_label.clone(), (point, poly_labels_set.clone()));
            point_gadgets.insert(point_label.clone(), (point_gadget.clone(), poly_labels_set));
        }

        let fs_seed = String::from("TEST_SEED").into_bytes();
        let mut fs_rng =
            PC::RandomOracle::from_seed(fs_seed.clone()).map_err(|err| PolyError::from(err))?;
        let proof = PC::multi_point_multi_poly_open(
            &ck,
            &polynomials,
            &points,
            &mut fs_rng,
            &rands,
            Some(rng),
        )?;

        let mut fs_rng =
            PC::RandomOracle::from_seed(fs_seed.clone()).map_err(|err| PolyError::from(err))?;
        let v_state = PC::succinct_multi_point_multi_poly_verify(
            &vk,
            &comms,
            &points,
            &values,
            &proof,
            &mut fs_rng,
        )?;
        assert!(v_state.is_some());

        let (vk_gadget, labeled_comms, mut fs_gadget) =
            alloc_gadgets_for_succinct_verify::<ConstraintF, G, PC, PCG, _>(
                cs.ns(|| "alloc gadgets for verify"),
                &vk,
                &comms,
                fs_seed,
                &test_conf.negative_type,
            )?;
        let proof_gadget =
            PCG::MultiPointProofGadget::alloc(cs.ns(|| "alloc proof gadget"), || Ok(proof))?;
        let _v_state = PCG::succinct_verify_multi_poly_multi_point(
            cs.ns(|| "verify proof"),
            &vk_gadget,
            labeled_comms.as_slice(),
            &point_gadgets,
            &value_gadgets,
            &proof_gadget,
            &mut fs_gadget,
        )?;
        let successful_test = cs.is_satisfied() ^ test_conf.negative_type.is_some();
        if !successful_test {
            println!("{:?}", cs.which_is_unsatisfied());
        }
        assert!(successful_test);
    }

    Ok(())
}

fn test_single_point_multi_poly_verify<
    ConstraintF: PrimeField,
    G: Group<BaseField = ConstraintF>,
    PC: PolynomialCommitment<G>,
    PCG: PolynomialCommitmentVerifierGadget<ConstraintF, G, PC>,
>(
    test_conf: TestInfo,
) -> Result<(), PCG::Error> {
    let rng = &mut thread_rng();
    for _ in 0..test_conf.num_iters {
        let (supported_degree, ck, vk) = setup_pc_parameters::<G, PC, _>(&test_conf, rng)?;

        let num_polynomials = rng.gen_range(1..=test_conf.num_polynomials);

        let mut polynomials = Vec::with_capacity(num_polynomials);
        let mut labels = Vec::with_capacity(num_polynomials);
        let mut values = Vec::with_capacity(num_polynomials);
        let point = G::ScalarField::rand(rng);
        for i in 0..num_polynomials {
            let label = format!("Test polynomial {}", i);
            labels.push(label.clone());
            let labeled_polynomial =
                generate_labeled_polynomial(test_conf.segmented, label, supported_degree, rng);
            values.push(labeled_polynomial.polynomial().evaluate(point));
            polynomials.push(labeled_polynomial);
        }

        let (comms, rands) = PC::commit_many(&ck, &polynomials, Some(rng))?;

        // alloc gadgets for polynomial evaluations over the point here as later on they will be moved in succinct verify function
        let mut cs = ConstraintSystem::<ConstraintF>::new(SynthesisMode::Debug);
        let mut value_gadgets = Vec::with_capacity(values.len());
        for (i, value) in values.iter().enumerate() {
            let value_gadget = NonNativeFieldGadget::<G::ScalarField, ConstraintF>::alloc(
                cs.ns(|| format!("alloc value for poly {}", i)),
                || Ok(value_for_alloc(value, &test_conf.negative_type, rng)),
            )?;
            value_gadgets.push(value_gadget);
        }

        let fs_seed = String::from("TEST_SEED").into_bytes();
        let mut fs_rng =
            PC::RandomOracle::from_seed(fs_seed.clone()).map_err(|err| PolyError::from(err))?;
        let proof = PC::single_point_multi_poly_open(
            &ck,
            &polynomials,
            point,
            &mut fs_rng,
            &rands,
            Some(rng),
        )?;
        let mut fs_rng =
            PC::RandomOracle::from_seed(fs_seed.clone()).map_err(|err| PolyError::from(err))?;
        let v_state = PC::succinct_single_point_multi_poly_verify(
            &vk,
            &comms,
            point,
            &values,
            &proof,
            &mut fs_rng,
        )?;
        if v_state.is_none() {
            Err(PolyError::FailedSuccinctCheck)?
        }
        let (vk_gadget, labeled_comms, mut fs_gadget) =
            alloc_gadgets_for_succinct_verify::<ConstraintF, G, PC, PCG, _>(
                cs.ns(|| "alloc gadgets for verify"),
                &vk,
                &comms,
                fs_seed,
                &test_conf.negative_type,
            )?;
        let proof_gadget = PCG::ProofGadget::alloc(cs.ns(|| "alloc proof"), || Ok(proof))?;
        let point_gadget = NonNativeFieldGadget::<G::ScalarField, ConstraintF>::alloc(
            cs.ns(|| "alloc point"),
            || Ok(point),
        )?;
        let _v_state_gadget = PCG::succinct_verify_single_point_multi_poly(
            cs.ns(|| "verify proof"),
            &vk_gadget,
            &labeled_comms,
            &point_gadget,
            &value_gadgets,
            &proof_gadget,
            &mut fs_gadget,
        )?;
        let successful_test = cs.is_satisfied() ^ test_conf.negative_type.is_some();
        if !successful_test {
            println!("{:?}", cs.which_is_unsatisfied());
        }
        assert!(successful_test, "failed for {:?}", test_conf.negative_type);
    }

    Ok(())
}

fn exec_test<FN: Fn(Option<NegativeTestType>)>(test_fn: FN) {
    test_fn(None);
    test_fn(Some(NegativeTestType::Commitments));
    test_fn(Some(NegativeTestType::Values));
}

pub(crate) fn succinct_verify_single_point_single_poly_test<
    ConstraintF: PrimeField,
    G: Group<BaseField = ConstraintF>,
    PC: PolynomialCommitment<G>,
    PCG: PolynomialCommitmentVerifierGadget<ConstraintF, G, PC>,
>() {
    exec_test(|negative_type| {
        test_succinct_verify_template::<ConstraintF, G, PC, PCG>(TestInfo {
            num_iters: 5,
            max_degree: None,
            supported_degree: None,
            num_polynomials: 1,
            max_num_queries: 1,
            segmented: false,
            negative_type,
        })
        .unwrap()
    })
}

pub(crate) fn succinct_verify_with_segmentation_test<
    ConstraintF: PrimeField,
    G: Group<BaseField = ConstraintF>,
    PC: 'static + PolynomialCommitment<G, Commitment = G>,
    PCG: 'static + PolynomialCommitmentVerifierGadget<ConstraintF, G, PC>,
>() {
    exec_test(|negative_type| {
        test_succinct_verify_template::<
            ConstraintF,
            G,
            DomainExtendedPolynomialCommitment<G, PC>,
            DomainExtendedPolyCommitVerifierGadget<ConstraintF, G, PC, PCG>,
        >(TestInfo {
            num_iters: 5,
            max_degree: None,
            supported_degree: None,
            num_polynomials: 1,
            max_num_queries: 1,
            segmented: true,
            negative_type,
        })
        .unwrap();
        test_single_point_multi_poly_verify::<
            ConstraintF,
            G,
            DomainExtendedPolynomialCommitment<G, PC>,
            DomainExtendedPolyCommitVerifierGadget<ConstraintF, G, PC, PCG>,
        >(TestInfo {
            num_iters: 5,
            max_degree: None,
            supported_degree: None,
            num_polynomials: 5,
            max_num_queries: 1,
            segmented: true,
            negative_type,
        })
        .unwrap();
        test_multi_point_multi_poly_verify::<
            ConstraintF,
            G,
            DomainExtendedPolynomialCommitment<G, PC>,
            DomainExtendedPolyCommitVerifierGadget<ConstraintF, G, PC, PCG>,
        >(TestInfo {
            num_iters: 5,
            max_degree: None,
            supported_degree: None,
            num_polynomials: 5,
            max_num_queries: 5,
            segmented: true,
            negative_type,
        })
        .unwrap()
    })
}

pub(crate) fn single_point_multi_poly_test<
    ConstraintF: PrimeField,
    G: Group<BaseField = ConstraintF>,
    PC: PolynomialCommitment<G>,
    PCG: PolynomialCommitmentVerifierGadget<ConstraintF, G, PC>,
>() {
    exec_test(|negative_type| {
        test_single_point_multi_poly_verify::<ConstraintF, G, PC, PCG>(TestInfo {
            num_iters: 5,
            max_degree: None,
            supported_degree: None,
            num_polynomials: 5,
            max_num_queries: 1,
            segmented: false,
            negative_type,
        })
        .unwrap()
    })
}

pub(crate) fn constant_polynomial_succinct_verify_test<
    ConstraintF: PrimeField,
    G: Group<BaseField = ConstraintF>,
    PC: PolynomialCommitment<G>,
    PCG: PolynomialCommitmentVerifierGadget<ConstraintF, G, PC>,
>() {
    exec_test(|negative_type| {
        test_succinct_verify_template::<ConstraintF, G, PC, PCG>(TestInfo {
            num_iters: 5,
            max_degree: None,
            supported_degree: Some(0),
            num_polynomials: 1,
            max_num_queries: 1,
            segmented: false,
            negative_type,
        })
        .unwrap()
    })
}

pub(crate) fn multi_poly_multi_point_test<
    ConstraintF: PrimeField,
    G: Group<BaseField = ConstraintF>,
    PC: PolynomialCommitment<G>,
    PCG: PolynomialCommitmentVerifierGadget<ConstraintF, G, PC>,
>() {
    exec_test(|negative_type| {
        test_multi_point_multi_poly_verify::<ConstraintF, G, PC, PCG>(TestInfo {
            num_iters: 5,
            max_degree: None,
            supported_degree: None,
            num_polynomials: 5,
            max_num_queries: 5,
            segmented: false,
            negative_type,
        })
        .unwrap()
    })
}

pub(crate) fn single_poly_multi_point_test<
    ConstraintF: PrimeField,
    G: Group<BaseField = ConstraintF>,
    PC: PolynomialCommitment<G>,
    PCG: PolynomialCommitmentVerifierGadget<ConstraintF, G, PC>,
>() {
    exec_test(|negative_type| {
        test_multi_point_multi_poly_verify::<ConstraintF, G, PC, PCG>(TestInfo {
            num_iters: 5,
            max_degree: None,
            supported_degree: None,
            num_polynomials: 1,
            max_num_queries: 5,
            segmented: false,
            negative_type,
        })
        .unwrap()
    })
}