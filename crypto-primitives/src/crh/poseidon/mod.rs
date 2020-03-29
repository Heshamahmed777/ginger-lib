extern crate hex;
extern crate rand;
extern crate rayon;

#[cfg(feature = "r1cs")]
pub mod constraints;

use rayon::prelude::*;

use algebra::fields::mnt6753::Fr as MNT6753Fr;
use algebra::fields::mnt4753::Fr as MNT4753Fr;
//use algebra::{PrimeField, SquareRootField, UniformRand, Field};
//use algebra::{Field, PrimeField, SquareRootField, UniformRand, Fp768};
//use std::ops::Mul;

//use std::time::Instant;

//use algebra::biginteger::BigInteger768;
//use algebra::{to_bytes, ToBytes};
//use algebra::field_new;

//use rand::SeedableRng;
//use rand_xorshift::XorShiftRng;

use crate::crh::poseidon::parameters::{MNT4753PoseidonParameters, MNT6753PoseidonParameters};
//use crate::crh::poseidon::poseidon_dual::poseidon_engine_dual;
//use crate::crh::poseidon::poseidon_original::poseidon_engine;
//use crate::crh::poseidon::poseidon_mt::poseidon_engine_gen;
//use crate::crh::poseidon::mul_inv::bench_mul_inv;
use std::marker::PhantomData;
//use crate::crh::Batched2to1CRH;
//use std::error::Error;

pub mod poseidon_original;
//pub mod poseidon_dual;
//pub mod poseidon_mt;
pub mod parameters;
//pub mod mul_inv;


pub trait PoseidonParameters: 'static + FieldBasedHashParameters{

    const T: usize;  // Number of S-Boxesb
    const R_F:i32;   // Number of full rounds
    const R_P:i32;   // Number of partial rounds
    const R:usize;   // The rate of the hash function
    const ZERO:Self::Fr;   // The zero element in the field
    const C2:Self::Fr;     // The constant 3 to add in the position corresponding to the capacity
    const AFTER_ZERO_PERM: &'static[Self::Fr]; // State vector after a zero permutation
    const ROUND_CST: &'static[Self::Fr];  // Array of round constants
    const MDS_CST: &'static[Self::Fr];  // The MDS matrix
    const MDS_CST_SHORT: &'static[Self::Fr];  // The MDS matrix for fast matrix multiplication

}

pub struct PoseidonCRH<P: PoseidonParameters> {
    parameters: PhantomData<P>

    //fn evaluate(parameters: &Self::Parameters, input: &[u8]) -> Result<Self::Output, Error> {}

}

//impl<P: PoseidonParameters> Batched2to1CRH for PoseidonCRH<P> {
//    const INPUT_NUM_PAIRS: usize = unimplemented!();
//    type Output = P::Fr;
//    type Parameters = P;
//
//    fn evaluate(input: &[u8]) -> Result<Self::Output, Error> {}
//
//}

//fn print_cst () {
//    let cst = Fr::from_str("3").map_err(|_| ()).unwrap();
//    println!("{:?}", cst);
//
//    let mut d_out = to_bytes!(cst).unwrap();
//    d_out.reverse();
//
//    println!("constant = {:?}", hex::encode(d_out));
//
//}

//fn print_cst () {
//    for i in 0..195 {
//        let cst = Fr::from_str(ROUND_CST[i]).map_err(|_| ()).unwrap();
//        println!("{:?}", cst);
//    }
//}
//
//fn print_mds () {
//    for i in 0..9 {
//        let cst = Fr::from_str(MDS[i]).map_err(|_| ()).unwrap();
//        println!("{:?}", cst);
//    }
//}
//



#[test]
fn test_poseidon_hash_mnt4() {

    type Mnt4PoseidonHash = PoseidonHash<MNT4753Fr, MNT4753PoseidonParameters>;

    let mut input = Vec::new();
    input.push(MNT4753Fr::from_str("1").unwrap());
    input.push(MNT4753Fr::from_str("2").unwrap());
    let output = Mnt4PoseidonHash::evaluate(&input);

    println!("{:?}", output);
}


#[test]
fn test_poseidon_hash_mnt6() {

    type Mnt6PoseidonHash = PoseidonHash<MNT6753Fr, MNT6753PoseidonParameters>;

    let mut input = Vec::new();
    input.push(MNT6753Fr::from_str("1").unwrap());
    input.push(MNT6753Fr::from_str("2").unwrap());
    let output = Mnt6PoseidonHash::evaluate(&mut input);

    println!("{:?}", output);
}

// #[test]
// fn test_mul_add() {
//
//     let num_rounds = 1000000000;
//     let seed = 128;
//
//
//
//     let mut carry = 0;
//     let k = 84;
//     let m = 123;
//     let mut v1 = 3;
//     let mut v2 = 3;
//     let mut v3 = 3;
//     let mut v4 = 3;
//     let mut v5 = 3;
//     let mut v6 = 3;
//     let mut v7 = 3;
//     let mut v8 = 3;
//     let mut v9 = 3;
//     let mut v10 = 3;
//     let now_mul = Instant::now();
//     for _i in 0..num_rounds {
//         v2 = mac_with_carry(v1, k, m, &mut carry);
//         v3 = mac_with_carry(v2, k, m, &mut carry);
//         v4 = mac_with_carry(v3, k, m, &mut carry);
//         v5 = mac_with_carry(v4, k, m, &mut carry);
//         v6 = mac_with_carry(v5, k, m, &mut carry);
//         v7 = mac_with_carry(v6, k, m, &mut carry);
//         v8 = mac_with_carry(v7, k, m, &mut carry);
//         v9 = mac_with_carry(v8, k, m, &mut carry);
//         v10 = mac_with_carry(v9, k, m, &mut carry);
//         v1 = mac_with_carry(v10, k, m, &mut carry);
//     }
//     let new_now_mul  = Instant::now();
//     println!("result = {}", v1);
//
//     let duration_mul =  new_now_mul.duration_since(now_mul);
//     println!("Time for {} rounds muladd = {:?}", num_rounds, duration_mul.as_micros());
//
//     let now_add = Instant::now();
//     for _i in 0..num_rounds {
//         v2 = adc(v1, k, &mut carry);
//         v3 = adc(v2, k, &mut carry);
//         v4 = adc(v3, k, &mut carry);
//         v5 = adc(v4, k, &mut carry);
//         v6 = adc(v5, k, &mut carry);
//         v7 = adc(v6, k, &mut carry);
//         v8 = adc(v7, k, &mut carry);
//         v9 = adc(v8, k, &mut carry);
//         v10 = adc(v9, k, &mut carry);
//         v1 = adc(v10, k, &mut carry);
//     }
//     let new_now_add  = Instant::now();
//     println!("result = {}", v1);
//
//     let duration_add =  new_now_add.duration_since(now_add);
//     println!("Time for {} rounds add = {:?}", num_rounds, duration_add.as_micros());
//
//
// }

#[test]
fn test_hash_speed() {

    type Mnt4PoseidonHash = PoseidonHash<MNT4753Fr, MNT4753PoseidonParameters>;
    type Mnt4BatchPoseidonHash = PoseidonBatchHash<MNT4753Fr, MNT4753PoseidonParameters>;

    //  the number of rounds to test
    let num_rounds = 1000;

    // the vectors that store random input data
    let mut vec_vec_elem_4753 = Vec::new();

    let mut array_elem_4753 = Vec::new();

    // the random number generator to generate random input data
    // let mut rng = &mut thread_rng();
    let mut rng = XorShiftRng::seed_from_u64(1231275789u64);

    // we need the double of number of rounds because we have two inputs
    for _ in 0..num_rounds {
        let mut vec_elem_4753 = Vec::new();
        let elem1 = MNT4753Fr::rand(&mut rng);
        let elem2 = MNT4753Fr::rand(&mut rng);
        vec_elem_4753.push(elem1.clone());
        vec_elem_4753.push(elem2.clone());
        vec_vec_elem_4753.push(vec_elem_4753);
        array_elem_4753.push(elem1.clone());
        array_elem_4753.push(elem2.clone());
    }

    // =============================================================================
    // Calculate Poseidon Hash for mnt4753
    let now_4753 = Instant::now();

    let mut output_4753 = Vec::new();

    for i in 0..num_rounds {

        // Call the poseidon hash
        let output = Mnt4PoseidonHash::evaluate(&vec_vec_elem_4753[i]);
        output_4753.push(output.unwrap());
    }

    let new_now_4753  = Instant::now();
    // =============================================================================

    // // =============================================================================
    // // Calculate Poseidon Hash for mnt4753 batch evaluation
    // let now_4753_batch = Instant::now();
    //
    // Mnt4BatchPoseidonHash::batch_evaluate(&mut array_elem_4753);
    //
    // // Call the poseidon batch hash
    // let mut output_4753_batch = Vec::new();
    //
    // for i in 0..num_rounds {
    //     output_4753_batch.push(array_elem_4753[i]);
    // }
    //
    // let new_now_4753_batch  = Instant::now();
    // // =============================================================================

    // =============================================================================
    // Calculate Poseidon Hash for mnt4753 batch evaluation

    let mut array1 = Vec::new();
    let mut array2 = Vec::new();
    let mut array3 = Vec::new();
    let mut array4 = Vec::new();

    for i in 0..(num_rounds/4) {
        array1.push(vec_vec_elem_4753[i][0].clone());
        array1.push(vec_vec_elem_4753[i][1].clone());
    }
    for i in (num_rounds/4)..(num_rounds/2) {
        array2.push(vec_vec_elem_4753[i][0].clone());
        array2.push(vec_vec_elem_4753[i][1].clone());
    }
    for i in (num_rounds/2)..(num_rounds*3/4) {
        array3.push(vec_vec_elem_4753[i][0].clone());
        array3.push(vec_vec_elem_4753[i][1].clone());
    }
    for i in (num_rounds*3/4)..(num_rounds) {
        array4.push(vec_vec_elem_4753[i][0].clone());
        array4.push(vec_vec_elem_4753[i][1].clone());
    }

    let mut array_array_input = Vec::new();
    array_array_input.push(array1);
    array_array_input.push(array2);
    array_array_input.push(array3);
    array_array_input.push(array4);


    let now_4753_batch = Instant::now();

    array_array_input.par_iter_mut().for_each(|mut p| Mnt4BatchPoseidonHash::batch_evaluate_2_1(&mut p));

    let new_now_4753_batch  = Instant::now();

    // Call the poseidon batch hash
    let mut output_4753_batch = Vec::new();

    for i in 0..num_rounds/4 {
        output_4753_batch.push(array_array_input[0][i]);
    }
    for i in 0..num_rounds/4{
        output_4753_batch.push(array_array_input[1][i]);
    }
    for i in 0..num_rounds/4{
        output_4753_batch.push(array_array_input[2][i]);
    }
    for i in 0..num_rounds/4{
        output_4753_batch.push(array_array_input[3][i]);
    }

    // =============================================================================

    // =============================================================================
    // Compare results
    let output_batch = output_4753_batch;
    for i in 0..num_rounds {
        if output_4753[i] != output_batch[i] {
            println!("Hash outputs, position {}, for MNT4 are not equal.",i);
        }
    }
    println!("End comparison for MNT4.");

    // =============================================================================
    // Report the timing results

    let duration_4753_single =  new_now_4753.duration_since(now_4753);
    println!("Time for {} rounds MNT4753 single = {:?}", num_rounds, duration_4753_single.as_millis());

    let duration_4753_batch =  new_now_4753_batch.duration_since(now_4753_batch);
    println!("Time for {} rounds MNT4753 batch = {:?}", num_rounds, duration_4753_batch.as_millis());

    // // =============================================================================
    // // =============================================================================
    //
    type Mnt6PoseidonHash = PoseidonHash<MNT6753Fr, MNT6753PoseidonParameters>;
    type Mnt6BatchPoseidonHash = PoseidonBatchHash<MNT6753Fr, MNT6753PoseidonParameters>;

    // the vectors that store random input data
    let mut vec_vec_elem_6753 = Vec::new();

    let mut array_elem_6753 = Vec::new();

    // the random number generator to generate random input data
    // let mut rng = &mut thread_rng();
    let mut rng = XorShiftRng::seed_from_u64(1231275789u64);

    // we need the double of number of rounds because we have two inputs
    for _ in 0..num_rounds {
        let mut vec_elem_6753 = Vec::new();
        let elem1 = MNT6753Fr::rand(&mut rng);
        let elem2 = MNT6753Fr::rand(&mut rng);
        vec_elem_6753.push(elem1.clone());
        vec_elem_6753.push(elem2.clone());
        vec_vec_elem_6753.push(vec_elem_6753);
        array_elem_6753.push(elem1.clone());
        array_elem_6753.push(elem2.clone());
    }

    // =============================================================================
    // Calculate Poseidon Hash for mnt4753
    let now_6753 = Instant::now();

    let mut output_6753 = Vec::new();

    for i in 0..num_rounds {

        // Call the poseidon hash
        let output = Mnt6PoseidonHash::evaluate(&vec_vec_elem_6753[i]);
        output_6753.push(output.unwrap());
    }

    let new_now_6753  = Instant::now();
    // =============================================================================

    // // =============================================================================
    // // Calculate Poseidon Hash for mnt4753 batch evaluation
    // let now_4753_batch = Instant::now();
    //
    // Mnt4BatchPoseidonHash::batch_evaluate(&mut array_elem_4753);
    //
    // // Call the poseidon batch hash
    // let mut output_4753_batch = Vec::new();
    //
    // for i in 0..num_rounds {
    //     output_4753_batch.push(array_elem_4753[i]);
    // }
    //
    // let new_now_4753_batch  = Instant::now();
    // // =============================================================================

    // =============================================================================
    // Calculate Poseidon Hash for mnt4753 batch evaluation

    let mut array1 = Vec::new();
    let mut array2 = Vec::new();
    let mut array3 = Vec::new();
    let mut array4 = Vec::new();

    for i in 0..(num_rounds/4) {
        array1.push(vec_vec_elem_6753[i][0].clone());
        array1.push(vec_vec_elem_6753[i][1].clone());
    }
    for i in (num_rounds/4)..(num_rounds/2) {
        array2.push(vec_vec_elem_6753[i][0].clone());
        array2.push(vec_vec_elem_6753[i][1].clone());
    }
    for i in (num_rounds/2)..(num_rounds*3/4) {
        array3.push(vec_vec_elem_6753[i][0].clone());
        array3.push(vec_vec_elem_6753[i][1].clone());
    }
    for i in (num_rounds*3/4)..(num_rounds) {
        array4.push(vec_vec_elem_6753[i][0].clone());
        array4.push(vec_vec_elem_6753[i][1].clone());
    }

    let mut array_array_input = Vec::new();
    array_array_input.push(array1);
    array_array_input.push(array2);
    array_array_input.push(array3);
    array_array_input.push(array4);


    let now_6753_batch = Instant::now();

    array_array_input.par_iter_mut().for_each(|mut p| Mnt6BatchPoseidonHash::batch_evaluate_2_1(&mut p));

    let new_now_6753_batch  = Instant::now();

    // Call the poseidon batch hash
    let mut output_6753_batch = Vec::new();

    for i in 0..num_rounds/4 {
        output_6753_batch.push(array_array_input[0][i]);
    }
    for i in 0..num_rounds/4{
        output_6753_batch.push(array_array_input[1][i]);
    }
    for i in 0..num_rounds/4{
        output_6753_batch.push(array_array_input[2][i]);
    }
    for i in 0..num_rounds/4{
        output_6753_batch.push(array_array_input[3][i]);
    }

    // =============================================================================

    // =============================================================================
    // Compare results
    let output_batch = output_6753_batch;
    for i in 0..num_rounds {
        if output_6753[i] != output_batch[i] {
            println!("Hash outputs, position {}, for MNT6 are not equal.",i);
        }
    }
    println!("End comparison for MNT6.");

    // =============================================================================
    // Report the timing results

    let duration_6753_single =  new_now_6753.duration_since(now_6753);
    println!("Time for {} rounds MNT6753 single = {:?}", num_rounds, duration_6753_single.as_millis());

    let duration_6753_batch =  new_now_6753_batch.duration_since(now_6753_batch);
    println!("Time for {} rounds MNT6753 batch = {:?}", num_rounds, duration_6753_batch.as_millis());



    //
    // // the vectors that store random input data
    // let mut vec_vec_elem_6753 = Vec::new();
    //
    // // the random number generator to generate random input data
    // // let mut rng = &mut thread_rng();
    // let mut rng = XorShiftRng::seed_from_u64(1231275789u64);
    //
    // // we need the double of number of rounds because we have two inputs
    // for _ in 0..num_rounds {
    //     let mut vec_elem_6753 = Vec::new();
    //     vec_elem_6753.push(MNT6753Fr::rand(&mut rng));
    //     vec_elem_6753.push(MNT6753Fr::rand(&mut rng));
    //     vec_vec_elem_6753.push(vec_elem_6753);
    // }
    //
    // // =============================================================================
    // // Calculate Poseidon Hash for mnt6753
    // let now_6753 = Instant::now();
    //
    // let mut output_6753 = Vec::new();
    //
    // for i in 0..num_rounds {
    //
    //     // Call the poseidon hash
    //     let output = Mnt6PoseidonHash::evaluate(&vec_vec_elem_6753[i]);
    //     output_6753.push(output.unwrap());
    // }
    //
    // let new_now_6753  = Instant::now();
    // // =============================================================================
    //
    // // =============================================================================
    // // Calculate Poseidon Hash for mnt4753
    // let now_6753_batch = Instant::now();
    //
    // // Call the poseidon batch hash
    // let output_6753_batch = Mnt6BatchPoseidonHash::batch_evaluate(&vec_vec_elem_6753);
    //
    // let new_now_6753_batch  = Instant::now();
    // // =============================================================================
    //
    // // =============================================================================
    // // Compare results
    // let output_batch = output_6753_batch.unwrap();
    // for i in 0..num_rounds {
    //     if output_6753[i] != output_batch[i] {
    //         println!("Hash outputs, position {}, for MNT6 are not equal.",i);
    //     }
    // }
    // println!("End comparison for MNT6.");
    //
    // // =============================================================================
    // // Report the timing results
    //
    // let duration_6753_single =  new_now_6753.duration_since(now_6753);
    // println!("Time for {} rounds MNT6753 single = {:?}", num_rounds, duration_6753_single.as_millis());
    //
    // let duration_6753_batch =  new_now_6753_batch.duration_since(now_6753_batch);
    // println!("Time for {} rounds MNT6753 batch = {:?}", num_rounds, duration_6753_batch.as_millis());
    //
    // // =============================================================================

}



// #[test]
// fn test_cst() {
//
//     //  the number of rounds to test
//     let num_rounds = 1000;
//
//     // the vectors that store random input data
//     let mut vec_elem_4753:Vec<MNT4753Fr> = Vec::new();
//     let mut vec_elem_6753:Vec<MNT6753Fr> = Vec::new();
//
//     // the random number generator to generate random input data
//     // let mut rng = &mut thread_rng();
//     let mut rng = XorShiftRng::seed_from_u64(1231275789u64);
//
//     // we need the double of number of rounds because we have two inputs
//     for _ in 0..(2*num_rounds) {
//         vec_elem_4753.push(MNT4753Fr::rand(&mut rng));
//         vec_elem_6753.push(MNT6753Fr::rand(&mut rng));
//     }
//
//     // =============================================================================
//     // Calculate Poseidon Hash for mnt4753 combining 2 hashes
//     let now_4753_dual = Instant::now();
//
//     let mut output_4753_dual = Vec::new();
//
//     for i in 0..num_rounds/2 {
//         let mut input1 = vec![vec_elem_4753[4*i], vec_elem_4753[4*i+1]];
//         let mut input2 = vec![vec_elem_4753[4*i+2], vec_elem_4753[4*i+3]];
//
//         // Call the poseidon hash
//         let output = poseidon_engine_dual::<MNT4753PoseidonParameters>(&mut input1, &mut input2);
//         output_4753_dual.push(output[0]);
//         output_4753_dual.push(output[1]);
//     }
//
//     let new_now_4753_dual  = Instant::now();
//     // =============================================================================
//     // =============================================================================
//     // Calculate Poseidon Hash for mnt4753 original
//     let now_4753_single = Instant::now();
//
//     let mut output_4753_single = Vec::new();
//
//     for i in 0..num_rounds {
//         let mut input1 = vec![vec_elem_4753[2*i], vec_elem_4753[2*i+1]];
//
//         // Call the poseidon hash
//         let output = poseidon_engine::<MNT4753PoseidonParameters>(&mut input1);
//         output_4753_single.push(output);
//     }
//
//     let new_now_4753_single  = Instant::now();
//     // =============================================================================
//
//
//     // =============================================================================
//     // Calculate Poseidon Hash for mnt6753 combining 2 hashes
//     let now_6753_dual = Instant::now();
//
//     let mut output_6753_dual = Vec::new();
//
//     for i in 0..num_rounds/2 {
//         //let mut input = Vec::new();
//         let mut input1 = vec![vec_elem_6753[4*i],vec_elem_6753[4*i+1]];
//         let mut input2 = vec![vec_elem_6753[4*i+2],vec_elem_6753[4*i+3]];
//
//         // Call the poseidon hash
//         let output = poseidon_engine_dual::<MNT6753PoseidonParameters>(&mut input1, &mut input2);
//         output_6753_dual.push(output[0]);
//         output_6753_dual.push(output[1]);
//     }
//     let new_now_6753_dual  = Instant::now();
//     // =============================================================================
//     // =============================================================================
//     // Calculate Poseidon Hash for mnt6753 original
//     let now_6753_single = Instant::now();
//
//     let mut output_6753_single = Vec::new();
//
//     for i in 0..num_rounds {
//         //let mut input = Vec::new();
//         let mut input1 = vec![vec_elem_6753[2*i],vec_elem_6753[2*i+1]];
//
//         // Call the poseidon hash
//         let output = poseidon_engine::<MNT6753PoseidonParameters>(&mut input1);
//         output_6753_single.push(output);
//     }
//     let new_now_6753_single  = Instant::now();
//     // =============================================================================
//
//     // =============================================================================
//     // Compare results
//     for i in 0..num_rounds {
//         if output_4753_dual[i] != output_4753_single[i] {
//             println!("Hash outputs, position {}, for MNT4 are not equal.",i);
//         }
//     }
//     println!("End comparison for MNT4.");
//     for i in 0..num_rounds {
//         if output_6753_dual[i] != output_6753_single[i] {
//             println!("Hash outputs, position {}, for MNT6 are not equal.",i);
//         }
//     }
//     println!("End comparison for MNT6.");
//
//
// //    // =============================================================================
// //    // Print the result of for mnt4753
// //    for i in 0..num_rounds {
// //
// //        // Reverse order to output the data
// //        let mut d_in_0 = to_bytes!(vec_elem_4753[2*i]).unwrap();
// //        d_in_0.reverse();
// //        let mut d_in_1 = to_bytes!(vec_elem_4753[2*i + 1]).unwrap();
// //        d_in_1.reverse();
// //
// //        let mut d_out_dual = to_bytes!(output_4753_dual[i]).unwrap();
// //        d_out_dual.reverse();
// //        let mut d_out_single = to_bytes!(output_4753_single[i]).unwrap();
// //        d_out_single.reverse();
// //
// //        println!("input[0] = {:?}", hex::encode(d_in_0));
// //        println!("input[1] = {:?}", hex::encode(d_in_1));
// //        println!("hash MNT4753 single = {:?}", hex::encode(d_out_single));
// //        println!("hash MNT4753 dual   = {:?}", hex::encode(d_out_dual));
// //
// //    }
// //    // =============================================================================
// //
// //    // =============================================================================
// //    // Print the result for mnt6753
// //    for i in 0..num_rounds {
// //        // Reverse order to output the data
// //        let mut d_in_0 = to_bytes!(vec_elem_6753[2*i]).unwrap();
// //        d_in_0.reverse();
// //        let mut d_in_1 = to_bytes!(vec_elem_6753[2*i + 1]).unwrap();
// //        d_in_1.reverse();
// //
// //        let mut d_out_dual = to_bytes!(output_6753_dual[i]).unwrap();
// //        d_out_dual.reverse();
// //        let mut d_out_single = to_bytes!(output_6753_single[i]).unwrap();
// //        d_out_single.reverse();
// //
// //        println!("input[0] = {:?}", hex::encode(d_in_0));
// //        println!("input[1] = {:?}", hex::encode(d_in_1));
// //        println!("hash MNT6753 single = {:?}", hex::encode(d_out_single));
// //        println!("hash MNT6753 dual   = {:?}", hex::encode(d_out_dual));
// //    }
// //    // =============================================================================
//
//     // =============================================================================
//     // Report the timing results
//
//     let duration_4753_single =  new_now_4753_single.duration_since(now_4753_single);
//     println!("Time for {} rounds MNT4753 single = {:?}", num_rounds, duration_4753_single.as_millis());
//
//     let duration_4753_dual =  new_now_4753_dual.duration_since(now_4753_dual);
//     println!("Time for {} rounds MNT4753 dual   = {:?}", num_rounds, duration_4753_dual.as_millis());
//
//     let duration_6753_single =  new_now_6753_single.duration_since(now_6753_single);
//     println!("Time for {} rounds MNT6753 single = {:?}", num_rounds, duration_6753_single.as_millis());
//
//     let duration_6753_dual =  new_now_6753_dual.duration_since(now_6753_dual);
//     println!("Time for {} rounds MNT6753 dual   = {:?}", num_rounds, duration_6753_dual.as_millis());
//
//     // =============================================================================
//
//     //bench_mul_inv();
//
// }

use std::str::FromStr;
use crate::crh::{FieldBasedHashParameters, FieldBasedHash, BatchFieldBasedHash};
use crate::crh::poseidon::poseidon_original::{PoseidonHash, PoseidonBatchHash};
use rand_xorshift::XorShiftRng;
//use algebra::{UniformRand, Fp768};
use algebra::UniformRand;
use self::rand::SeedableRng;
use std::time::Instant;
//use algebra::biginteger::arithmetic::{mac_with_carry, adc};
//use crate::crh::{FieldBasedHashParameters, FieldBasedHash};
//use crate::crh::poseidon::poseidon_original::PoseidonHash;

#[test]
fn print_cst_mnt4_ord_4_57_4() {
    let cst0=MNT4753Fr::from_str("10884172332582723603619715255165090438714684550677381657882587110402038601385043396730434822037485320065292160074806567155416459857762592098221716837854333806215598888315779501649044582384758858042968061088791534420624114121100").map_err(|_|()).unwrap();
    println!("{:?}", cst0);
    let cst1=MNT4753Fr::from_str("28464951170107685184940274196098437155252777358248469690874618821915589973886122582351849811854696408699799086208568855093168576583749363214918968345130357228989661887009752572368193494099416878229608057579935455872809222728987").map_err(|_|()).unwrap();
    println!("{:?}", cst1);
    let cst2=MNT4753Fr::from_str("27075394508109978866654764988600247821899413612604873677869138587750524163718648289053978621420832046210456286210044682619526426388824500874543505904513088321365378547529792201941788321038201335544860034829839036932251473499151").map_err(|_|()).unwrap();
    println!("{:?}", cst2);
    let cst3=MNT4753Fr::from_str("18004137732202961453791373987395049024084373958131760038919446184877961041708807598680155908806942625084357175623061245866060965832812139111415230459492039162791124605018310393828598643761089379142614201996249517185560683276768").map_err(|_|()).unwrap();
    println!("{:?}", cst3);
    let cst4=MNT4753Fr::from_str("2074551474966452637925040396421734087813491280692221695780368451757366632383027012022017973219188984799467506572780848230349165194182427168901698523586673471001683876918289560756983762228830396262009839769197951722897480272899").map_err(|_|()).unwrap();
    println!("{:?}", cst4);
    let cst5=MNT4753Fr::from_str("15069732132331910631230479210378887506036913700885784696753316318261221828206867080385700093141684748060252805187789996765290267426415589984847782866434915105721177815132576388948163318647651062672392476175253736397398239874244").map_err(|_|()).unwrap();
    println!("{:?}", cst5);
    let cst6=MNT4753Fr::from_str("27666003282516952314556189026976067483248080020789813751919196906747199559718300906981360260263512864048349266681045978381022084757085546251556957997376356445597760369950154324639376245004709219436458071649656547008272367598967").map_err(|_|()).unwrap();
    println!("{:?}", cst6);
    let cst7=MNT4753Fr::from_str("25023581200734435821437622360933873881277057801266470970408671195571384564097469761121473896449105703172967159787982858879765274233253531004894160277470810624867227185600160508067651811283990972472834140041626299063702445967289").map_err(|_|()).unwrap();
    println!("{:?}", cst7);
    let cst8=MNT4753Fr::from_str("4301590533758743077967535853270350171474461560524818121363033530718760842754035703339856173965255916071722819708478625915147917877542763754260197318476682752339126894745337355953371082904121300242153548111779618404119249337724").map_err(|_|()).unwrap();
    println!("{:?}", cst8);
    let cst9=MNT4753Fr::from_str("41591888675708955397989521229835315678977922791968682696908032659086639654916058063032203992526281439319074321760379729920029064994725351617909687359896459904948072711279716310310382864927007568361716481947225633964881856515550").map_err(|_|()).unwrap();
    println!("{:?}", cst9);
    let cst10=MNT4753Fr::from_str("8191021785513454498891420454673963633823624202575816683305628939565196149900132526604703292733253308181771424821985203561957195034430559747487400596878275531756640874230787983761398570306063882314421079611685010854186627416322").map_err(|_|()).unwrap();
    println!("{:?}", cst10);
    let cst11=MNT4753Fr::from_str("37433068301589928065171818460723175509058263333602272501317253117225891962712808809027228060717921039906961725083790737919498877809233493886454752989758354978599658208763406670546907785470402077569329531248774699476227304880553").map_err(|_|()).unwrap();
    println!("{:?}", cst11);
    let cst12=MNT4753Fr::from_str("34258232143310556937684898289243159363814288022975732148858319367692349829885071984406888509272237525746636528777346476181884154768663898420135717534609855995519616993465205806168028619928398114336244151943142868590877686724691").map_err(|_|()).unwrap();
    println!("{:?}", cst12);
    let cst13=MNT4753Fr::from_str("25758497809833149795784640631076342916234832687469911597235418036972360603371126086347174983183404839890591199869916927687302803211348409352447027519665970771925202215960910535591239500945621503423660667881371513169945044637105").map_err(|_|()).unwrap();
    println!("{:?}", cst13);
    let cst14=MNT4753Fr::from_str("17484487464089075389831956282547686710038292486863129523637293528942162408359867526364728537195450132857676036224697167762056020620122827895997181505632020152022643798525041134119184127481829306351705745594360943027101978984664").map_err(|_|()).unwrap();
    println!("{:?}", cst14);
    let cst15=MNT4753Fr::from_str("26728978244776975416479579053893321238602467708028391849633406095696993905443704150183774244178058344775502250431078772784674981428201042764213038571579503849709048454785095440502502342431595450798597647970342061092440123192468").map_err(|_|()).unwrap();
    println!("{:?}", cst15);
    let cst16=MNT4753Fr::from_str("26731196467226840197589706514728160495526703443741870899204924450667069628407267742339468824911435606402861861582380340468434806238326201555948267292888750197725552800465641231613077056008378100414001630019652885416482080626343").map_err(|_|()).unwrap();
    println!("{:?}", cst16);
    let cst17=MNT4753Fr::from_str("28854551565493226215230083337728822286818273034634545407991105726272222957366243986863455489571961262626294849383188238314538370731027847431435613956587019240348931321130207873547632275146625575381609712638156871530676032674985").map_err(|_|()).unwrap();
    println!("{:?}", cst17);
    let cst18=MNT4753Fr::from_str("41062236497541421947388562620003616622223580845764580397095478600639524076477672317492646956634598413661395043305525385463011086466829041219689788517139951168638469696533272346229947329347339224483935216941974200270368057107024").map_err(|_|()).unwrap();
    println!("{:?}", cst18);
    let cst19=MNT4753Fr::from_str("27938895169345136667971220382782975560074009272866767494584221841834619856474641615038297485524802476816171994365221958329966080763385554722577951905578333894827601529525723625872746997342244891264464063997309872079818740571610").map_err(|_|()).unwrap();
    println!("{:?}", cst19);
    let cst20=MNT4753Fr::from_str("38289489590135728890317091053498834433872306544041756731663394888954987540921593785410599094172734168545590846649399635017134124622801871007266137371172839592468555803208328802318115565841454888098188105079863102249756774116838").map_err(|_|()).unwrap();
    println!("{:?}", cst20);
    let cst21=MNT4753Fr::from_str("21212460178042659256135530771269624745686669166090806088526715124510769643347347291617454004648303231899432292475000985876497053108570948338634947269402053317905852933974023445660595136641700524021016039408441649713009160072238").map_err(|_|()).unwrap();
    println!("{:?}", cst21);
    let cst22=MNT4753Fr::from_str("31686182428191424985362670948535364808695585675968271003538850962254553524225678303471653005384720764225410878666117387597839467036785052292355721822463378876731348303996456786067598971917982987037114701817193025931113568630150").map_err(|_|()).unwrap();
    println!("{:?}", cst22);
    let cst23=MNT4753Fr::from_str("40809162069631660207902802475136615829687633643340159905129095011921411325328929191944243727631689732046859810979222311841309982221576932327037859861494443280481340343865607760875296014732969262601881696583090667897781185719322").map_err(|_|()).unwrap();
    println!("{:?}", cst23);
    let cst24=MNT4753Fr::from_str("433416243928738642484894608432960894807074303424813457938400014208224479362209140633868400958955453122155867314768010272472819072598011908410313540974890456944656398086358581351116587549750856095147592695988007362974006542107").map_err(|_|()).unwrap();
    println!("{:?}", cst24);
    let cst25=MNT4753Fr::from_str("33163007194547572532878026524366130601090505981832504585932233330942900850190759748322273198972209028449459582441118004240881918835299692514858689123033307089360631050729247601428012297768725710732196627259477350566380768228365").map_err(|_|()).unwrap();
    println!("{:?}", cst25);
    let cst26=MNT4753Fr::from_str("21183666258758571969903020154590380365324000944796008972178576572297501638052278136255537262141483047909931462230062129462281960850552282421551678839043126334676561291394869160362728573646020012062046763800724568853396935334250").map_err(|_|()).unwrap();
    println!("{:?}", cst26);
    let cst27=MNT4753Fr::from_str("2251800089314088982860795467661922266666502198201586874578336991330455192400665834901786283522299109996105339892043867578385531616378549639988097620121265806415066820540738152445660465483440000993843950865983729383276087553920").map_err(|_|()).unwrap();
    println!("{:?}", cst27);
    let cst28=MNT4753Fr::from_str("40612418632893654618381181665707101772005528525663033530749536928545979444698596352555764299100451272284937125226324771276771034813297378457492997138581986102620522082179279241525613758699505295856115181816057666917443793922201").map_err(|_|()).unwrap();
    println!("{:?}", cst28);
    let cst29=MNT4753Fr::from_str("6379459039534500678785210193334655097216718262865525979267995643252334551475988666617797564269969485630174274840031843009714240356206872132551255524182917701458442652553463143551068616434029366196306993643616038551844053246360").map_err(|_|()).unwrap();
    println!("{:?}", cst29);
    let cst30=MNT4753Fr::from_str("30468027773012812900633042433702175403902785972206189934490295978961913851789873848877400597291651220155573954345968293092515807603064524420003324288294178433025162088828630362567592132152695320378039897997702473682529687774171").map_err(|_|()).unwrap();
    println!("{:?}", cst30);
    let cst31=MNT4753Fr::from_str("18465196816946894465292980430915596567009345889578399051371349819186118498870841488514140420076612786473262042030761137618133995902069376633097829091307178519299838965454380597418914973600115446107190867277192333158988335176416").map_err(|_|()).unwrap();
    println!("{:?}", cst31);
    let cst32=MNT4753Fr::from_str("2402281393809604758741861634102320250039076085238471041563360295349254388767869366783607845420379016702258147303465139486192243397624262015651507032498506409826736602471549484257957540975285817517504865296391127500167414234719").map_err(|_|()).unwrap();
    println!("{:?}", cst32);
    let cst33=MNT4753Fr::from_str("38611498943235712991662859447625266963535012758595109162911530560668717256145767991029067742323713097814309256216632693291597805325871735601849051469071709849244476140569078358258017804935194704209278373667918084345502801835545").map_err(|_|()).unwrap();
    println!("{:?}", cst33);
    let cst34=MNT4753Fr::from_str("3563630882748448654041978424351912381598048508736661918898310116159782086507471234003394848124257923783216305599988829094191232982382208545747045652002475708501652769726929662931537273228051226097495225148676954285029768588515").map_err(|_|()).unwrap();
    println!("{:?}", cst34);
    let cst35=MNT4753Fr::from_str("13919669789731812687504319926321869013430541983316941142235484345428429134856795404079950090003197912724319649523014943494875474944413292862090817097108215036473631276243451171906740057737692495357387682506895823339939146323578").map_err(|_|()).unwrap();
    println!("{:?}", cst35);
    let cst36=MNT4753Fr::from_str("7341670737157617918643923819313973498634546031568667237391857084404922731651446100085614660195041883457602545851904502852421799321176476839733730809297352064512307120854633336002289890489577886317608702175608617630378639736796").map_err(|_|()).unwrap();
    println!("{:?}", cst36);
    let cst37=MNT4753Fr::from_str("18375625437249523119283305187515709238987699104291926533592799158364430489349315909716470900271742571215117303479891763714154696588868343004582405475186558054057933179145261046158127852979221813620666836480210524519717325188901").map_err(|_|()).unwrap();
    println!("{:?}", cst37);
    let cst38=MNT4753Fr::from_str("41229978522507793843112532040924003956113066110493030762125180007472841942183917563847110567352315775185524085204458723294539299756449274308954181339484158430741861177107409600740739165031543743116122459794015048357243038501910").map_err(|_|()).unwrap();
    println!("{:?}", cst38);
    let cst39=MNT4753Fr::from_str("2006358263522607648951338753854742367384422180995965302069921107436145238366964174590846372458731071531371851784713472661591879347137251379080960997873613912871479227913155207692598235889168861905689460103335502153729384121235").map_err(|_|()).unwrap();
    println!("{:?}", cst39);
    let cst40=MNT4753Fr::from_str("20945454620202163303234084442434317621730284336053115507979943839261926137944518628612273379368434230317423043978451332354758242190707854874819850046452542805087635851135240237324873889378734689485251384246362759536038428091825").map_err(|_|()).unwrap();
    println!("{:?}", cst40);
    let cst41=MNT4753Fr::from_str("31844831535804527593944240244201455558305485299781626678866794012155031685613782405060026873882253344036637808290258588522021853934889766535005609434976621931800856449297936381058480590926874852587740057081590074238870792770037").map_err(|_|()).unwrap();
    println!("{:?}", cst41);
    let cst42=MNT4753Fr::from_str("2852693811136890047390660519948585361768006641403980455315873139559211135877690328232379675906312771051893546911449434416342650804737366429066925488146799345878545899845747959041521122999598964321804366569566748441445750139318").map_err(|_|()).unwrap();
    println!("{:?}", cst42);
    let cst43=MNT4753Fr::from_str("9085555982519629067991555048234442118124640033613110243792243700321177309717220509514083558122756779617610381593679796054383958815720918979559302475786049019552939135137185724319812724114642479060304516959228039030418570655509").map_err(|_|()).unwrap();
    println!("{:?}", cst43);
    let cst44=MNT4753Fr::from_str("12626006824930768980002106628472089131680692227636569711799115118178564604056556994078083121378547990439234720198145185440573390302212314294119314402531452267173148308327260115103053090413616056544620538843934667053950043985371").map_err(|_|()).unwrap();
    println!("{:?}", cst44);
    let cst45=MNT4753Fr::from_str("38164969896840956823767941250548088435547933120338251156904320802308303575363496918269555639109591276367390019998812219603892142175740823897090184005735607340115206821661287333855264879696231399685944176809565116982621570637296").map_err(|_|()).unwrap();
    println!("{:?}", cst45);
    let cst46=MNT4753Fr::from_str("17090213495452708274004717870631087367903793359533765028403431525832560520797105890803084087054140326958515276281555615207807976901392504063328299668425638835949624835622963041640178829660563022505706505527806240579826956084519").map_err(|_|()).unwrap();
    println!("{:?}", cst46);
    let cst47=MNT4753Fr::from_str("9067558931657594830057894758390031162549132674249862581780587376315696299780073801431713555104978909537830465527200522270279276807879775780797885352478917110434702420826163460955081264740102177754173208673244509548417139066951").map_err(|_|()).unwrap();
    println!("{:?}", cst47);
    let cst48=MNT4753Fr::from_str("40032963253309610899620441667212330015511861094928701091329259812008467690224110525490093528409974222652625041368066066046941426017066102952100390886101451645722257653079783230510570357979705190715706520389420294503924573306778").map_err(|_|()).unwrap();
    println!("{:?}", cst48);
    let cst49=MNT4753Fr::from_str("3816684805160639422225670519063414294328370985573312078247233974530966746533721933800931954675104071268893052732797317350795823398913243694403048328176604755173456613792775150841658681985378093002674253439779128235797932926726").map_err(|_|()).unwrap();
    println!("{:?}", cst49);
    let cst50=MNT4753Fr::from_str("3360702466782198287842424143635875179880180471348359856415682607910322774648535184405308162977399695947931182352654571151721218426947461315809903973280275926501158988979642457742097551947138767547160102492293367886134769164497").map_err(|_|()).unwrap();
    println!("{:?}", cst50);
    let cst51=MNT4753Fr::from_str("1240591046071790742878494101655034891303584415172610313038289523179506984130913769693571093630445934930341843409538195399048065672502471724740904734590847104919851989161148295180256544241076926560748419098404758445410929118691").map_err(|_|()).unwrap();
    println!("{:?}", cst51);
    let cst52=MNT4753Fr::from_str("9518791513245169654335590074428700713877816644305288341008350906888669367796657479255003175176261058755979545308881809755220424899982668344041171788584250140604084380477711565052222109172835022976699017487395270094718974835228").map_err(|_|()).unwrap();
    println!("{:?}", cst52);
    let cst53=MNT4753Fr::from_str("23399397495852097088566857659627425420438231743037209632526828699713779989058112499807967735613985094259876197331274202989958432273937133977308341467073715750905183854045115732580532318919642310496227825560873128612997701001008").map_err(|_|()).unwrap();
    println!("{:?}", cst53);
    let cst54=MNT4753Fr::from_str("39795820862558309225240317549208128469141828376475020537284104852259018678731830355134478445536149937336211391565941608345036196677763942362294769218523206643799718226343522622136062700275148113177047280998475440454177122349325").map_err(|_|()).unwrap();
    println!("{:?}", cst54);
    let cst55=MNT4753Fr::from_str("343973013406998212235641205544613387488053556879633052554732012913451043062424838376876789677296116323306066174106094461390345579673645885548478072441447183280755987231802508553620832547434541554335254727522389927014097426881").map_err(|_|()).unwrap();
    println!("{:?}", cst55);
    let cst56=MNT4753Fr::from_str("35384867649961873021140626479477022844972875318820765662731985772458452771494670345862348499610527115632749854355088795343331363070919121092537591702056127848011851281772069378619257990584537253621630721322359558147465627356699").map_err(|_|()).unwrap();
    println!("{:?}", cst56);
    let cst57=MNT4753Fr::from_str("24020394736204271882279998279742615394853070630959187404989410189905490687181320825951935977478389005283176708321600421304494146520185587453687011900845381688150813254122420607542993525252384579776579876807435209076395114494031").map_err(|_|()).unwrap();
    println!("{:?}", cst57);
    let cst58=MNT4753Fr::from_str("2391214998247129072007663350029426370656309562675960355748979774337394089263321728611667387725777525236868195703192864045803789358407516178685186936633385691497356734179229439044975265665878139379307366031444765568150149976453").map_err(|_|()).unwrap();
    println!("{:?}", cst58);
    let cst59=MNT4753Fr::from_str("7960310035956100286492304036802641412504334429692032477465160540371806218037840278952253550925921294703129502062519165888717372843212436114425308823506911304808658809391176736541820788612995643821346073415091568808461472184023").map_err(|_|()).unwrap();
    println!("{:?}", cst59);
    let cst60=MNT4753Fr::from_str("18397782660099010748149053196027287995643889310234868291209660788281435074009378186188711143802455031022962584234918865589303304976799672792429061603140897481353598250872121218378129495050685379274080882227575143930675144329908").map_err(|_|()).unwrap();
    println!("{:?}", cst60);
    let cst61=MNT4753Fr::from_str("28404728317966071573309991090472635119438913074756352428768776485950466302361934186228020296248090666236388386136605147332210644066059123045869014128322894157933009431135289916888091900373548535502997046134682073717131367919497").map_err(|_|()).unwrap();
    println!("{:?}", cst61);
    let cst62=MNT4753Fr::from_str("40337820476891437015177730790804292132672007461815502038589406260559844440634200611522944686371789060428229498706050678373884567403062056104296196063237610932098405282952512507706481485042460581063632855541395142700523708781049").map_err(|_|()).unwrap();
    println!("{:?}", cst62);
    let cst63=MNT4753Fr::from_str("4994013974544604367456422488903681885667499970788140440515846193846975504453542509283207194391383474376582233466081568579153679972629834008843533951593132878128522776269756709487741704675052287837217218537177653119919827857144").map_err(|_|()).unwrap();
    println!("{:?}", cst63);
    let cst64=MNT4753Fr::from_str("925329506828444318806094396709940736618229326164472766944397647097990861168800049599692926038821653367856289276154949197466672693760979168012166201638203436329247992697617172868332550162883445814294738531542458701655535384616").map_err(|_|()).unwrap();
    println!("{:?}", cst64);
    let cst65=MNT4753Fr::from_str("38005992986633364535602707286384291757214593818968991987854508148811942591809260390437432745604083177143527896620585492543931727122935748144836194317836535031945774211264903102017340923038465419719073835517082358463626547384858").map_err(|_|()).unwrap();
    println!("{:?}", cst65);
    let cst66=MNT4753Fr::from_str("41663769033318000537936454408132177031165323582511427539099695105648899467092376843742012196039559765296043704170640847682290358580870919693685431232943009710275314852503460454891537218311897780251600158348615721230301132882617").map_err(|_|()).unwrap();
    println!("{:?}", cst66);
    let cst67=MNT4753Fr::from_str("22963411308290006291448053485448904855267029021652920527156730509909884173252599865995558763153806578388041439469599279619231696939346317625878421759070665104825420475767031865205006256650507879769766352111056751903387387875764").map_err(|_|()).unwrap();
    println!("{:?}", cst67);
    let cst68=MNT4753Fr::from_str("9312729778061011136055680420800555676955503843783944363288079304732227611686335681791999565002080299987426707672922938673707621512243485799874774130996529606151018355131894303488872527828156588683057653647587501121181637209766").map_err(|_|()).unwrap();
    println!("{:?}", cst68);
    let cst69=MNT4753Fr::from_str("13384972323574951035100669568409549313604837336697431114958589619341987317989488871685498791284883454788049443310016694028985493034024205098207119593803766275262365246940427700973244450547254242346333812828099590989684447680960").map_err(|_|()).unwrap();
    println!("{:?}", cst69);
    let cst70=MNT4753Fr::from_str("34219284672629318505799132212769465144698763766524958477086497661554965759533045941307744662432344864669617877330199048836261244467804331948977738648971409622455495505665822287631245279108332189480939499006618100493067892242495").map_err(|_|()).unwrap();
    println!("{:?}", cst70);
    let cst71=MNT4753Fr::from_str("5529097128889294025450244231685504776042574373512369607641886747093963673679679998021636282729987177190568735980622035918689072446897062192219154428853639826480343217825589803353082442301343730382302314582214237017932334107289").map_err(|_|()).unwrap();
    println!("{:?}", cst71);
    let cst72=MNT4753Fr::from_str("3559098952174469226791848674825554594361893842650684078548230288457305227971488406319793164704956718976210364456353792703104685224338821020102328122521694966471585081910659264977313670485695911087402265742951999556382331863279").map_err(|_|()).unwrap();
    println!("{:?}", cst72);
    let cst73=MNT4753Fr::from_str("29053077012833777264101650135077750120778486124855972690376143385869439526550148268117354253143501893030168762424262542484456655324817612411278129226741227117980329301597576379088263073514301534821090265795023984053214329657659").map_err(|_|()).unwrap();
    println!("{:?}", cst73);
    let cst74=MNT4753Fr::from_str("23009039804709756781467374031993425177068364392025466078470742577188988926583008268124231941814637632345480539445211664079113796495130079942448446707809095929120468994031721218517026543886462789660293858995912632062665166150637").map_err(|_|()).unwrap();
    println!("{:?}", cst74);
    let cst75=MNT4753Fr::from_str("6072882053958134211095672142729452576182520065512054211825936238787122166734439220093524967231480013539577802894348190003674938790440986775163579258514444109876081300577111221213540467758839214733532618443880905805732073484303").map_err(|_|()).unwrap();
    println!("{:?}", cst75);
    let cst76=MNT4753Fr::from_str("35059193330280539463391973499574319082005113849322935358863742752442524815844538563434911473587143375637984175939071494092539302133812504790186003576762624358853662282709064282590576095086048785853253147595250199805363023436776").map_err(|_|()).unwrap();
    println!("{:?}", cst76);
    let cst77=MNT4753Fr::from_str("18931400505736160376843565532474069500917586173651066289550284460787397006406484690808691491966943871535089909520657776788812825110124814990741579135684477053804110977305109190740780128163637304605369322446259708833241802476057").map_err(|_|()).unwrap();
    println!("{:?}", cst77);
    let cst78=MNT4753Fr::from_str("40869605384780498625663472259003464148169768612844081303378060006679339178280297072259858001509758404966990899903664009215814736891328573305801103327453085753156234923538073939726045052878794643774479669503470499348995476752456").map_err(|_|()).unwrap();
    println!("{:?}", cst78);
    let cst79=MNT4753Fr::from_str("27618156430404699976666867453253723177391880380420633979209660702825647728834293905324842620913728489537354538492257168090890977088909705791460266927347459960957410119788967276777939160106321216199435811609936518053291659933499").map_err(|_|()).unwrap();
    println!("{:?}", cst79);
    let cst80=MNT4753Fr::from_str("27029433955898717996923454382501274989289484538722163898705550948776278839180524114634554444535280150700417975719359077986395461374426131435322923452936900888705208754597521403793689287966004870141316627443389181462120570599758").map_err(|_|()).unwrap();
    println!("{:?}", cst80);
    let cst81=MNT4753Fr::from_str("35370563451118060289751627530705513730465472472916289424479471700282936860980356177122768338401489998199899902516905817569712456373183519432187223714241175410729499098562368073454329275562499162752895409961842925683497317397855").map_err(|_|()).unwrap();
    println!("{:?}", cst81);
    let cst82=MNT4753Fr::from_str("3759419852644648642738618787559153094017762489121177542046546290639043744827088535130727642292267641560077121916948210153391972986595396622175296123889156954776485499540266143978759710949411488403233909522715373073895974687780").map_err(|_|()).unwrap();
    println!("{:?}", cst82);
    let cst83=MNT4753Fr::from_str("17649848295039720612395370346397529406795208430546735294422826923787582032762407840725855744062099076357164867597994030363525423458788932110000644845816530323261326682930130034044723246225184634461271227207970467671603840325572").map_err(|_|()).unwrap();
    println!("{:?}", cst83);
    let cst84=MNT4753Fr::from_str("27800604408873464587550376926779919904795374865342095960361264871509068970969511303751445022108299494721040932286707582681624301370070839004155090631744827551008576900122364583327432574720067418003652422691582941653184621780806").map_err(|_|()).unwrap();
    println!("{:?}", cst84);
    let cst85=MNT4753Fr::from_str("28385174298739780473994905940787069035720305905305604794273460743650142421438299126059342936706040843026889401820964023557175612704835588311492581561682449281364518713683426530177749433250722376224543939968267975064487872214971").map_err(|_|()).unwrap();
    println!("{:?}", cst85);
    let cst86=MNT4753Fr::from_str("24769623226540071529279229825455924506263538975251524976229779737027809451111227186631954771269999174278677800356910160061311481151924817574489107044581903873582897329740201781176985913235446879653470431094060068683608561604274").map_err(|_|()).unwrap();
    println!("{:?}", cst86);
    let cst87=MNT4753Fr::from_str("1649244793395247361460518565124951401244687125233845957097580927691811014820465282004236750347053320275201944921055571906992527391799247391437964906547774275042141047524426611334352713265650427934158759329240264690562788135426").map_err(|_|()).unwrap();
    println!("{:?}", cst87);
    let cst88=MNT4753Fr::from_str("5462205734697092513081389848651161686987276796672153535423877444305277524402748663520931704101090629919097275409316669036782627125048976941158878815842240847555864177566920610804118245910368616323258503415315212124387633253057").map_err(|_|()).unwrap();
    println!("{:?}", cst88);
    let cst89=MNT4753Fr::from_str("40667365437579061928406992446737629856705406914548035630826262931959738126544119217251060000388557805656227873189278296451867872181867271992062698275323515331243454777196249564255097062540012424685066806084203776417664633324193").map_err(|_|()).unwrap();
    println!("{:?}", cst89);
    let cst90=MNT4753Fr::from_str("26418071738221983555944967632169518595344152231967756879120335032923106706639391405712718901477786032331583810861691122509278956625581349842086309828914884015902523899045339317656583353331635558811422224431771712731661130843016").map_err(|_|()).unwrap();
    println!("{:?}", cst90);
    let cst91=MNT4753Fr::from_str("15355553127167236075734277519100239877471360062996938582135296728416476107476407024870771009128481911307944546388485657943447235104038610178929525142469022387607061884806148585343383550757961550733983990585785561017207100488407").map_err(|_|()).unwrap();
    println!("{:?}", cst91);
    let cst92=MNT4753Fr::from_str("17291286679129551759637959142952029835130056232930700865825472874847584880585768812767870618244482635667555331174164257791883506492545201919766605969947059933059853095352711875544673371486512208989109771662632924364880319564628").map_err(|_|()).unwrap();
    println!("{:?}", cst92);
    let cst93=MNT4753Fr::from_str("27287212707878768848131775179088698771497497844307543848648567273335343343084556255681317476994998956600596408783326915185488295533946927789764547034123949021218147841037051062461389186361823379179725662487752184142385766173967").map_err(|_|()).unwrap();
    println!("{:?}", cst93);
    let cst94=MNT4753Fr::from_str("5292817510486493318824620057280575295405885292922640481427264491901721519657060965249971582570885403133441698023552325012083003658040647851723502234927684264349641075992244648265675549783354196979111035733802450742175436528170").map_err(|_|()).unwrap();
    println!("{:?}", cst94);
    let cst95=MNT4753Fr::from_str("22721549989485385608231317301124416826244128972892531347281346841721015353922917739073898502683539457989534041449091298358902563319730293847759571245401671582704062846665768186891591103313570462101642710064051710959329876373819").map_err(|_|()).unwrap();
    println!("{:?}", cst95);
    let cst96=MNT4753Fr::from_str("37468687658905773496023085425448714304130788226150878672499313009132951884437814931365346714196494918284011978235616924010770389372883605482351950048416922116529212026295953824795097090523643250553024430319894510227144059883691").map_err(|_|()).unwrap();
    println!("{:?}", cst96);
    let cst97=MNT4753Fr::from_str("38998729351119566340974178421050791884621673789002113043623998353176836031187812588401305016172736912009828288475905807676504095418270322776636858396732716464129011426087886264313849045424421420336763944574811808575654559076376").map_err(|_|()).unwrap();
    println!("{:?}", cst97);
    let cst98=MNT4753Fr::from_str("5106770189441456601114041609464551243264472537271482983527856305225171868611847752355400553975376204720978076321744531136758353387789097515252428435191897829429746261914197275900689637818543100184319302317595635229925934002873").map_err(|_|()).unwrap();
    println!("{:?}", cst98);
    let cst99=MNT4753Fr::from_str("5772458197491366042741652879265449326224046191019903671600690727800083759338667466092929026541920190436930342185692897764570589834144601111712904600710357200470797303132451548265534236791627885122931703140438864070522490659779").map_err(|_|()).unwrap();
    println!("{:?}", cst99);
    let cst100=MNT4753Fr::from_str("4436919442672844552815677954499160559813084935684065999462328173154215906541759404095446638413369699498505806562053381427168001506795100080886873093568523597023321691394950399380567211010729801250612423596194615762424950607022").map_err(|_|()).unwrap();
    println!("{:?}", cst100);
    let cst101=MNT4753Fr::from_str("8733456295148069332388754417809462364081007109251741732870032412471808800899281535604513913569523437014794459674459160473276654860381424250321734013420355880983580149357670332933470890048347737304794616489498031919474030435810").map_err(|_|()).unwrap();
    println!("{:?}", cst101);
    let cst102=MNT4753Fr::from_str("23200839689765896674333649674346591184942145275696212320889830758658231455721781654706561312136657147344246737772835947694856124792263773291223253977853436007708604825977331618388515179588045871284903310785172979392823834889638").map_err(|_|()).unwrap();
    println!("{:?}", cst102);
    let cst103=MNT4753Fr::from_str("6139239490757783300967837000869040266881617255999720362916193014012412040347227859238277100194050151520812882059062309106463395839700507519777381538184458974620221449410070218463437293691449772530163031840275396621239659725982").map_err(|_|()).unwrap();
    println!("{:?}", cst103);
    let cst104=MNT4753Fr::from_str("2491373639305848003940225632975515874464438102306429391457063338415295692417046800278627595554034141406456352734998225891553138910341738732371305359759782978258915765729138799505044418837429488058464433679924591698959469350325").map_err(|_|()).unwrap();
    println!("{:?}", cst104);
    let cst105=MNT4753Fr::from_str("29975900874338933548135531646527918085575401970156958729628404751594943635968466998994026132469000395332847451879199916246442454814311191180385972595760189442411317632961195679079006501599813529024381029391347294697414419563264").map_err(|_|()).unwrap();
    println!("{:?}", cst105);
    let cst106=MNT4753Fr::from_str("9077213191902191045616472958027138381694653022589245017004905102205191575242513150576272546408397705502195660810278657942841979024006727244406755048060469370400014682671935064813707604344363202987489026072405168249927803027627").map_err(|_|()).unwrap();
    println!("{:?}", cst106);
    let cst107=MNT4753Fr::from_str("23514579149565323197812429033364453635049741199435223135523978290455886757169172354017112688235486860861918834738911158817156022378409204317152383469626419201411397539038589908847712809060086749222647214068953703295232555702954").map_err(|_|()).unwrap();
    println!("{:?}", cst107);
    let cst108=MNT4753Fr::from_str("14539572607896281156712984164720370675254436264176285748156454225785741826386566556623817056033074179653214436425605072383035633353185891280843685384880418501675776187972499033820416336575918838982814587406147682269798551832364").map_err(|_|()).unwrap();
    println!("{:?}", cst108);
    let cst109=MNT4753Fr::from_str("41621992842849973022615763393755188364164034961042210231820675215736806494042942615455236585182054126344442337247927107052994211357529757719382949883060779360484804623403308669438783159496747087544203801838746042341389986893006").map_err(|_|()).unwrap();
    println!("{:?}", cst109);
    let cst110=MNT4753Fr::from_str("13671389221402636890551115748332315477305491724350897162100016192026750790194376432752298524847571716915410110234176354660321312759311250801048076612483911140072633074076832967504999698714395295211416881177425024864335165949466").map_err(|_|()).unwrap();
    println!("{:?}", cst110);
    let cst111=MNT4753Fr::from_str("18321240387341160769772888550556526304766997469029059325707497081959693281106614968991235373681951294148810010566391044457607569019177805949568010905932297468036471468878730100788396687130557540907975530678436160460874384337781").map_err(|_|()).unwrap();
    println!("{:?}", cst111);
    let cst112=MNT4753Fr::from_str("359037671350280874091164424758074960455399389608179531826922613098663082819529228869819339411802077964186384880137786611284990241304439102850186784277273125553847077365928252691826804337752915261126123263443821284571871157646").map_err(|_|()).unwrap();
    println!("{:?}", cst112);
    let cst113=MNT4753Fr::from_str("28271032157012524289158615819145696524953030828767350547825172998098923239495858719257044391778215736155884738438050732017021406009404077606890699541629605761918125588013433620683814609404574476420837852040996183687463221607078").map_err(|_|()).unwrap();
    println!("{:?}", cst113);
    let cst114=MNT4753Fr::from_str("35338344167633570434945136281469102648642870217389296647056442184016912733482015657234537431354420041610641652920487536081378477278213554806518526503080647891722622207324199403664988762359352009316475211762504040151544006169423").map_err(|_|()).unwrap();
    println!("{:?}", cst114);
    let cst115=MNT4753Fr::from_str("31229326810811447927803138343640795462379490065071204822704556936493863207663730228010352403371898358733364973488437514137694219157931331642520525758306553913894534495618927785815397775133892819214036825510363131945689678494924").map_err(|_|()).unwrap();
    println!("{:?}", cst115);
    let cst116=MNT4753Fr::from_str("27430135821884207674249369635195115291212928707214092631221669238074466830577124453398063292745367499868499612444421718782228612045879601477659124267592100510960236115131368461900125083819220482098165203154002378714835510094512").map_err(|_|()).unwrap();
    println!("{:?}", cst116);
    let cst117=MNT4753Fr::from_str("882186244015979503709726632079756956025635724503901978105716502857805196556767554760370796176378281776702379535108021318763984753575452466990260172552323074498347872787971916217140687980267187086584421439538261407884359761958").map_err(|_|()).unwrap();
    println!("{:?}", cst117);
    let cst118=MNT4753Fr::from_str("18465631499729468994899207903557135761212399766906896900879800774056717071606711817706251318312805490716892828478455449739514693869667652388418675682897030580094054875951867496686750976679918227467041718100843184103445491754310").map_err(|_|()).unwrap();
    println!("{:?}", cst118);
    let cst119=MNT4753Fr::from_str("38108017790996440560411285111393759927795989031764865751930841372094539329043272920764054986451039365332346846183185384270754394349201888325839974375099337843198096732584263707759629489091068284798609599529047250839287519264781").map_err(|_|()).unwrap();
    println!("{:?}", cst119);
    let cst120=MNT4753Fr::from_str("22387651162873028578832094941085976569582110562576402395460665203425797910219320952138483179888854417805338497258050731944546121802035690344883782900031950892246042622716375977126790500513827118292632388542424097136376850906480").map_err(|_|()).unwrap();
    println!("{:?}", cst120);
    let cst121=MNT4753Fr::from_str("34930415933068259910035552659837876652574555684422400726752377661366914353445079772796749672896822461904602091694873409746017528516794884157259666391310156786754265471880934253504903669050163146863901483803773446254615103601922").map_err(|_|()).unwrap();
    println!("{:?}", cst121);
    let cst122=MNT4753Fr::from_str("2963952542867525790804556111391556252952572012993292354383115748224205561399477545818705821789067340433134022761948622332336083777866112467968779172275452457623590968182731371559285411007533617539807856446543791015544021039169").map_err(|_|()).unwrap();
    println!("{:?}", cst122);
    let cst123=MNT4753Fr::from_str("10783062955607074050336370836330017725775773690984698391444720044145331733936307323976977519059710964928115176953665852412633565088947710926672049289214878852424241278303216222331698739011732348691376548087295967067485992769286").map_err(|_|()).unwrap();
    println!("{:?}", cst123);
    let cst124=MNT4753Fr::from_str("13956434888389825677955309453509560689047872307697551456366512730915940050244204814882505141551620711257155129783008727436029392561890447681063426009935890643674541075981449861824545586784211807159969492791388203768534953961274").map_err(|_|()).unwrap();
    println!("{:?}", cst124);
    let cst125=MNT4753Fr::from_str("33408642403564050158704957144044018273746922182052709330687284953723714398159890469356704279493602110020084847267860516332746349801823366572766772942246776224560932174346519480745712203938794427161790057096832284977986140596589").map_err(|_|()).unwrap();
    println!("{:?}", cst125);
    let cst126=MNT4753Fr::from_str("35386440609670012098806210455817530970601171767418135174216011008898103366937857773491811316473861227609846770201450926439445768520414504912648511821059995645380470228584620169345137763417810928680213050206315178464936891582209").map_err(|_|()).unwrap();
    println!("{:?}", cst126);
    let cst127=MNT4753Fr::from_str("34655272786623892713637181347214033199703926857057399779657254211944364671595542379010873506410373406312370474633795615468724121097646067100013815982605944512674497770457148108413297448677192053662526197589620426486689966500061").map_err(|_|()).unwrap();
    println!("{:?}", cst127);
    let cst128=MNT4753Fr::from_str("2815367302271340286284984059917317725892665164261117929947373995642966068800680924821426303776400213771736577431414782902290191616456860895771878780854990992009162730570982818189366019761005097939568610157896945354237150860350").map_err(|_|()).unwrap();
    println!("{:?}", cst128);
    let cst129=MNT4753Fr::from_str("1813635891024914032663478984644947594951439847308861456446354519075172761376495058921448997223872779995482607013382094243045089541569452757143641005534355019775454073291894367125953286951502453068024661772664306071646352058626").map_err(|_|()).unwrap();
    println!("{:?}", cst129);
    let cst130=MNT4753Fr::from_str("10725801573494508922404281779891002820275724952721397645343928717329072266575228399074350166484134908357606804765215428604505288414304818108590774167978050790977505269093375378880235817502127063195225760144052733111487839975650").map_err(|_|()).unwrap();
    println!("{:?}", cst130);
    let cst131=MNT4753Fr::from_str("30995288214914472870430042218301319817279035016122771512504687117871081590413798246635979087517049640249359501245360599006309685616737058616986580439975716312859289952977027928696366364995655715187954403179830497524535448277346").map_err(|_|()).unwrap();
    println!("{:?}", cst131);
    let cst132=MNT4753Fr::from_str("3400817640164393499098906942498601944897234538635997357831080571360925146798222260868201109864015226209818375430290395881140829061184497768633718859590962028691212897940582650517844774797994064367459644670940718954084052247975").map_err(|_|()).unwrap();
    println!("{:?}", cst132);
    let cst133=MNT4753Fr::from_str("3456290928443470566403053177247638550070253048906624499907008089502009113371834667633595291661084269009388614878685823555751169198428422096694483737281355985334103430077717021292824540027774496154374739310285057931585044219523").map_err(|_|()).unwrap();
    println!("{:?}", cst133);
    let cst134=MNT4753Fr::from_str("23597680584558953132299262990988710233971770050617246498655559295097049721392560016470829983398536863553622519520588737175227241556665617461562702989087786603847250170231030074366235263438154759072454754092129456577097203799139").map_err(|_|()).unwrap();
    println!("{:?}", cst134);
    let cst135=MNT4753Fr::from_str("28070654851220132884093003682663351523234352403162076063522436025076826139641291157976449124491913861584166859049404037820370758670421999414646872800299003355401586068633337103490964476307981258966296431375660175865256470559337").map_err(|_|()).unwrap();
    println!("{:?}", cst135);
    let cst136=MNT4753Fr::from_str("15266562189191074948070909534161031167320036598149332161960067428031197715976531901853534609839532054978727774131606314263014613896293159231797389830730326347005466135146897507356837219452350557476317666840672810448726758342489").map_err(|_|()).unwrap();
    println!("{:?}", cst136);
    let cst137=MNT4753Fr::from_str("8957472034141758958762281473979067723887986288846852582829114423383361266744813008701617234048046181047328627730413839369636509323684658155364126134462543650244934623681613244820038899379013363431116397416487128306696431298889").map_err(|_|()).unwrap();
    println!("{:?}", cst137);
    let cst138=MNT4753Fr::from_str("27357349691045496694528062984777744411456118783154730126566973683408848989502256217004852220031687837335783686293418853361199412094791999137530226867237094201006719932644797179136280948486180050441610967562980954391023291330506").map_err(|_|()).unwrap();
    println!("{:?}", cst138);
    let cst139=MNT4753Fr::from_str("34571137620535034672276934934238007265883388447877284021069747111122046890665812516929001314212765067123926695876001869776502991908546079857934716147556064568036116944539158795223174453849428493566125530124759615845609524562373").map_err(|_|()).unwrap();
    println!("{:?}", cst139);
    let cst140=MNT4753Fr::from_str("16381748891450110507264761691889036297789700702434085983944982274338523632195080187582793727934250193417638281505450122318836218350454283964427497503801211621236325071996175290787025869765295763526463762041521673966641957155705").map_err(|_|()).unwrap();
    println!("{:?}", cst140);
    let cst141=MNT4753Fr::from_str("6021257288967061898210148199845828532934422421715438150442174796434984441462923873319345258882019400171071706821995032856689475044411014382236741714270341220536743166006600191635580521140484505170182908558047831136699801676679").map_err(|_|()).unwrap();
    println!("{:?}", cst141);
    let cst142=MNT4753Fr::from_str("22459811406205694910254288451999428249399595564678156378254730818547314262666716190275188182443884925583498522014627188483144534941278748954517646372668869314339478397440223946874720257748269263229359323715353149445981588638341").map_err(|_|()).unwrap();
    println!("{:?}", cst142);
    let cst143=MNT4753Fr::from_str("40973409157784302372636777391240757747133596016803734351097594533715039479923264515079186587280401644941459394303298306229821151453849705462374531088311231194067905548495550385393351710938381222987295916491398766989803010834840").map_err(|_|()).unwrap();
    println!("{:?}", cst143);
    let cst144=MNT4753Fr::from_str("17508843160415478571745763084056297383101913963550614845803039033213447568992867745464068075404678026393201893387865724647507343720486745176923542590917515592176732807183318236825767754447731543004672433126854100638132353677889").map_err(|_|()).unwrap();
    println!("{:?}", cst144);
    let cst145=MNT4753Fr::from_str("39068485502543258580361392185836036112500393039459261071816850991279713543114716224875787886013807336426619612474032586552681157017319281039757166479977829983799474338052114358682600291098854045368113885558732388915517483194196").map_err(|_|()).unwrap();
    println!("{:?}", cst145);
    let cst146=MNT4753Fr::from_str("2712274030451320276781680505895113460199345022325775374064498971573960706641115129562908694493352355434622436990686676357360733953622426158953552415025278380863399194435146148372159371653131798943452292909107790666698272276166").map_err(|_|()).unwrap();
    println!("{:?}", cst146);
    let cst147=MNT4753Fr::from_str("38553290035651999397941538838959622576112409232015751905622752754376117226959367141016961872653056350439944719076394881405911559557624584444264236128284750066810895349556621326684193576850117100170270075771330491000193687907575").map_err(|_|()).unwrap();
    println!("{:?}", cst147);
    let cst148=MNT4753Fr::from_str("39544066702301491990725886994262307779395824868980874256862568308799222383647297693645579389335532478574358004185589029180752490946908938328634678016350256164152595868098825828331009951674941573631249554464306612893690009796416").map_err(|_|()).unwrap();
    println!("{:?}", cst148);
    let cst149=MNT4753Fr::from_str("35078743348264718446027399061897302009362274483723275149650767979839005172891453279673595484822042037395533490046637586055375147678291356904686664594023765468853119576475685536849008129121773655785072985856523002741370019031886").map_err(|_|()).unwrap();
    println!("{:?}", cst149);
    let cst150=MNT4753Fr::from_str("14835682520744163537708469539402200320286535290485327503225331859722494790449811886134652652545095725958961725580835441131117540300057924151884240609000501894829625894088716407937918997515744395029641286127402426254552532077350").map_err(|_|()).unwrap();
    println!("{:?}", cst150);
    let cst151=MNT4753Fr::from_str("4999447914525987443908637182497339717888642419276669596378301053493740060635509545074182954724637353643744879155523218940091965591105653526033665595719702542461676533181063858724045841591384614920488428055395042941472517865457").map_err(|_|()).unwrap();
    println!("{:?}", cst151);
    let cst152=MNT4753Fr::from_str("7784302244700206073876829054783223969300266564885122664269183972427840089354634950262004928993472504786480473336264234348505219803620576934188746206321848196278894119118404103438108368025245327251485421833335018764838492077155").map_err(|_|()).unwrap();
    println!("{:?}", cst152);
    let cst153=MNT4753Fr::from_str("37751104942314138189614473020847493413406753765134724420563438689176641023423229421996046438452482458708838452171777586096481313330295901112870746687391770800078574147903630132242215621424274692556363982364670302427874706606692").map_err(|_|()).unwrap();
    println!("{:?}", cst153);
    let cst154=MNT4753Fr::from_str("10406060713199885621068493803238442352460731914372457036984729565620602717371989710673257706122234962308833027084398518924894650712106972798574545664766700431591195636213717414080923209366957046117804583530038720106431668350827").map_err(|_|()).unwrap();
    println!("{:?}", cst154);
    let cst155=MNT4753Fr::from_str("2593464135822131454599895154278584008595083357262319513729127173335045730317102600830929472972680634411269223404784176372247951428991009641102551010971620970167965939277248670492952788381476094374439282456102096818264054362526").map_err(|_|()).unwrap();
    println!("{:?}", cst155);
    let cst156=MNT4753Fr::from_str("23383536409944103190102224622226558322599759826535651584483519813490230838809282758921089697283973525798323359139140807250280290921314970333946927316943093057713006710363446531311063649902183941049747199243159965480784168041237").map_err(|_|()).unwrap();
    println!("{:?}", cst156);
    let cst157=MNT4753Fr::from_str("7325172926494266061640790319848229789187409574408759947674852185678652628413063768108795522381008737469921494535131498051283700863865444256953610539482322283389136756622719541808654250107529029959101178080202500095293162622319").map_err(|_|()).unwrap();
    println!("{:?}", cst157);
    let cst158=MNT4753Fr::from_str("29749494666887335297667411948109890098282716117689073907272181874959678012865598392833930429652130777023936323731025522278345903734743964850028927194850095037060683385110445321159189486788423831382081704348452228878877613649795").map_err(|_|()).unwrap();
    println!("{:?}", cst158);
    let cst159=MNT4753Fr::from_str("7026075826232763636203067226373964457928636764650188420256312967195507351421094456481712746065774466437051657989809660485162141344037811685798104098630896385087146393854767655140668063019010171132370394292290215349945247455681").map_err(|_|()).unwrap();
    println!("{:?}", cst159);
    let cst160=MNT4753Fr::from_str("3487711351193332300614603050615464265881038448697414412798180490474793664266274828737000888196019726227780608455995838540763057983033205809942306646714921022341659825454769334335663485923570810384072595549915856664107214034500").map_err(|_|()).unwrap();
    println!("{:?}", cst160);
    let cst161=MNT4753Fr::from_str("5185507632461039110648075310583892012139490182677403845755296933267815246759417538579232302981518796576472950973751388084485609047062727349718616929136120791748654805541532082082197470108956764962478995256501848583017920463232").map_err(|_|()).unwrap();
    println!("{:?}", cst161);
    let cst162=MNT4753Fr::from_str("18398137257008986506390612383680680913530623840403454302452080252001263623241303927786025519016274717414685413180931421145268766065051582563105068949730401013621193347110024052539620577280522994931774253709750777406562850161897").map_err(|_|()).unwrap();
    println!("{:?}", cst162);
    let cst163=MNT4753Fr::from_str("32951162640339566634705265877785781505620506975386947165321309846369555276179552727535886598387511880470440141144582086744495806900753280758532810044238348537999214142894365635433389501334429671207187229774240124874382160965168").map_err(|_|()).unwrap();
    println!("{:?}", cst163);
    let cst164=MNT4753Fr::from_str("16342924274113546427721589297234537782564940594860686053132850200632685391962837074319771079203147071249761466043819000319471346195757251108714470854610389237572750238304859031566552731846927926195421425180949383322704130128116").map_err(|_|()).unwrap();
    println!("{:?}", cst164);
    let cst165=MNT4753Fr::from_str("24527511259200539443483061302405376345328539672896345620714672827253949390578981461668458893789010969893823824125417967783538956689659706698967427576820369935177256129136931546617199945322047679844726257628015199542929712274819").map_err(|_|()).unwrap();
    println!("{:?}", cst165);
    let cst166=MNT4753Fr::from_str("17090735537455347508137384534681818632990518829934464455577218938584878262413846104152265663918151209585122422515065830976075115000862441281130217740695200428628881230285284369768957946608766507250143480334060281043924554770563").map_err(|_|()).unwrap();
    println!("{:?}", cst166);
    let cst167=MNT4753Fr::from_str("40142458859842890306279772690449165574407566441332983298568885552000334216821022164225551960678565031631653974748215616727201477084225734894944804900649743120804758499876667773442587601269866597901040495692763999685235196842295").map_err(|_|()).unwrap();
    println!("{:?}", cst167);
    let cst168=MNT4753Fr::from_str("17569805745702648494170451259649507888191689637989292234473975603009388354913985789722522431395290516639446208550864900060274835025204093851712159638260587494378583607947820902841512758327170916760328212250640683520483369310696").map_err(|_|()).unwrap();
    println!("{:?}", cst168);
    let cst169=MNT4753Fr::from_str("8809409173777473832427875258053505436759670225660455295507016482519361737454807124096736567234953598977259275399143882860014988675515513739425140044396945237469447070671909619231367089735369896234953805503266461887200397836307").map_err(|_|()).unwrap();
    println!("{:?}", cst169);
    let cst170=MNT4753Fr::from_str("831160117079487725857625527835319325069547324504848476069765856104501131173036435830048336515893355425737353075048774082904712437005754039162308621534954176605013934724116808576424087117596408903480931245375500265497770631401").map_err(|_|()).unwrap();
    println!("{:?}", cst170);
    let cst171=MNT4753Fr::from_str("1064902420172598216339151721618705640008343984047092842779512780645057741423355739130511589004340755999147567542194071176646362491584368207085688601987755612214701767630761903398198733983473326182957924826974766252840225008054").map_err(|_|()).unwrap();
    println!("{:?}", cst171);
    let cst172=MNT4753Fr::from_str("27529863325189936771457710624122722313061593511149301901343788230536929562788172112760838243084392575993136775200356206218479280216777573819511670314415259406251289216992188527961263419847528891144137990032806926087604120576223").map_err(|_|()).unwrap();
    println!("{:?}", cst172);
    let cst173=MNT4753Fr::from_str("12988854247127162534338994230058089755229714115410943066081495425709559355506957886745918817472973927437027389439217422631003172244646031109318604212030381907958905263467209226604623672637552864472392033663042553608837626187358").map_err(|_|()).unwrap();
    println!("{:?}", cst173);
    let cst174=MNT4753Fr::from_str("40954368653134052496558443077127262203955740537940849322260783247485323908225221168185742684750276155726078815305460803668456051579263337892156039599957331536849136842836018776440224713172028293072349786629044251799136645796858").map_err(|_|()).unwrap();
    println!("{:?}", cst174);
    let cst175=MNT4753Fr::from_str("25345763513858299481251459076550381423935013580546383114176028187340774175589891044931926470117092414743511859490323434756119900559328412046829341343141300904114184947661598999423654189932341068002099406644571561888975613572265").map_err(|_|()).unwrap();
    println!("{:?}", cst175);
    let cst176=MNT4753Fr::from_str("32462409324774776336172576254782277269437048857856917865684363057722863957126485180263684895937732047890767074657492264367223402016583056821652252322898273512828316279021690191016674868789239421988240639502449574062855320134009").map_err(|_|()).unwrap();
    println!("{:?}", cst176);
    let cst177=MNT4753Fr::from_str("28477053716679587906610306010157357641614917933941602528855842879117969327241169620676859865594500272456558359177439714487508201874059315257250123811523115620513964555961649067764562457971535407713691605010794920855925100004971").map_err(|_|()).unwrap();
    println!("{:?}", cst177);
    let cst178=MNT4753Fr::from_str("25358467344601605954858604620006349332658815129899806144595126760006532686190898684831252633402493536011418648189738918148192950751115227340070881705891008428139430538081978422372506846645216003570428920901103223554548729335373").map_err(|_|()).unwrap();
    println!("{:?}", cst178);
    let cst179=MNT4753Fr::from_str("23973047647133230426325699331066798677390618436650291282144952821038675557592847228913261820563904161273172374586534139269530727361015103498727876874330412305748313471703059784567635125100090388875251714166654647289866980299577").map_err(|_|()).unwrap();
    println!("{:?}", cst179);
    let cst180=MNT4753Fr::from_str("8229226075629114561870972289965366697739428764301675206077660661850816045534290194004421673182929708266297278296648055140252924271385395734036798040586457665801022828617411190911711724501528611353619917440191708064090800329863").map_err(|_|()).unwrap();
    println!("{:?}", cst180);
    let cst181=MNT4753Fr::from_str("34352720670164404874823144638005609702464030901857096910132560068933608469548598098064107854304780488151976206488042657453795117943220667661404538407481675297687784146541411069249074335235300207815029769246670203001417702913827").map_err(|_|()).unwrap();
    println!("{:?}", cst181);
    let cst182=MNT4753Fr::from_str("28109920641758980157969404665431978595386851483458883945135977846168911980017242088332245328530742731980721161557322545943761517352109980236193912090024011407283724944734632918545865109161391980231864049997243669719648515298881").map_err(|_|()).unwrap();
    println!("{:?}", cst182);
    let cst183=MNT4753Fr::from_str("3545374976656149822324079127889463687720407404909372027950799573116624881981293255965828773589781984025305753133020862434001792340316775954768711812952882061130565873984627426075362520730314688574082619706484729783214629060680").map_err(|_|()).unwrap();
    println!("{:?}", cst183);
    let cst184=MNT4753Fr::from_str("23931533963595714851493320590771440269083938785619091024026825006274965235998325695163660519675335410720487491254789285808181930733011998409631718003898452551923853640744584985143199544402934897420723638061829110035998077424998").map_err(|_|()).unwrap();
    println!("{:?}", cst184);
    let cst185=MNT4753Fr::from_str("9786921476633594290177279710042130488580198826507133277191790898271643640218840911923823114552779341728419729696287583947727513224291033882255390771949086906278288474428047136194956028429577995522774527206852321136785381766162").map_err(|_|()).unwrap();
    println!("{:?}", cst185);
    let cst186=MNT4753Fr::from_str("4572792515919992213334301263558092995383716737708745486919332457290481983430992324071113112483459770621531467051442604901175063203717541518506598688345276244010494269901091484550452486132406291126852175392030192107148207296061").map_err(|_|()).unwrap();
    println!("{:?}", cst186);
    let cst187=MNT4753Fr::from_str("4885231286596695572338828657675444503141256324216350271527406872437966109482841732365400351279363584535381721754568551160513345498371785143783143568493883948163427931699725351705044462905741471233922195267287054020623409406990").map_err(|_|()).unwrap();
    println!("{:?}", cst187);
    let cst188=MNT4753Fr::from_str("32113869291898317303431045757153303246042980446190258902835091274831409401610324730119115167617599681099133342558686844530292063234713410580444101269511764730702679945966761224773359731549313643485576938854472176490523859679252").map_err(|_|()).unwrap();
    println!("{:?}", cst188);
    let cst189=MNT4753Fr::from_str("16070660516524471718521427900264258113099730083020975335802806373584498200867879803862304638697604308754620220457088529393137897617917882814085902302422664102685489059458832587849890423943097946807552409305383798680447332277678").map_err(|_|()).unwrap();
    println!("{:?}", cst189);
    let cst190=MNT4753Fr::from_str("35308851747935072521147968797210376602268827199384620273548945511819006694891283869024262760243839211733129333772591203408300345429493537816453582045873241381888883593545082530209444715371173976365195351370701979551122284919236").map_err(|_|()).unwrap();
    println!("{:?}", cst190);
    let cst191=MNT4753Fr::from_str("34705573003707084157816530535141433653702835279551711916390638261171025702694459231066944815518200504382438667059275277443358834102611916973027441875632209445105737641444740819504450172291803282944120450990798173748832223665339").map_err(|_|()).unwrap();
    println!("{:?}", cst191);
    let cst192=MNT4753Fr::from_str("17980997210561515426166317666768611171867307933089816096282220816656973721485455760079081187182805326433477838551824014381531517996810266496471001151601105152412517652149567034004974875479580439949529549648449085733087404431765").map_err(|_|()).unwrap();
    println!("{:?}", cst192);
    let cst193=MNT4753Fr::from_str("3751317812694215823863229816110765625666455820069509186559057434675868003698960652950725893035933892790997611052135528373148006136762922478002196403059400124033746219876331634140189242725113532035127886763802076129786648576216").map_err(|_|()).unwrap();
    println!("{:?}", cst193);
    let cst194=MNT4753Fr::from_str("8530788221311884474527514966709494470282854188656402697517726609054780743717528379497471080927021753641558010880415259639134618920250392065400968023192200874464814474144862360444264769007160309158166191631396906711575955898282").map_err(|_|()).unwrap();
    println!("{:?}", cst194);
}


#[test]
fn print_cst_mnt6_ord_4_57_4() {
    let cst0 = MNT6753Fr::from_str("16977013836989733012056609122489142600730823881415065271241911317399234646909461032933775818888557121369698369709543084399248436315606617522564244422473249783099561491916332951710962859069415557089416220802577440786749776501229").map_err(|_|()).unwrap();
    println!("{:?}", cst0);
    let cst1 = MNT6753Fr::from_str("6514840401823895236012039130739102814102291787469760954108097596594037776319879898765280027211139382908249487460870271745529079897929246138456154555137991855698127349252143392690212902650000578871337039843858392094365702341848").map_err(|_|()).unwrap();
    println!("{:?}", cst1);
    let cst2 = MNT6753Fr::from_str("28877059783967413361049121815791122444256735086718804936110537148050489131974610623362746556562457709257741783779473835692078595406895835625690362363072958528536345309991459140061683618431327344335211094017079346082956969190135").map_err(|_|()).unwrap();
    println!("{:?}", cst2);
    let cst3 = MNT6753Fr::from_str("12681517565470201445993096179467103314320785212518365624076503182479842014302669424760874702975503872811007617476143749555812609312260667333320921283332461188787116893290255210421090650291538725540462321289224643761350288077351").map_err(|_|()).unwrap();
    println!("{:?}", cst3);
    let cst4 = MNT6753Fr::from_str("31445448972620351607950542287318885355035420661600932506277104142836439587105538001061256780427206862460569573652241825324130317234997524690809474714015262986541057755166920121745859572192681046948767697694345614896389224402352").map_err(|_|()).unwrap();
    println!("{:?}", cst4);
    let cst5 = MNT6753Fr::from_str("18192974414699449640237099229982425094905214066170745393138058661203761733947168638422317827403348488101597842509740492677371282353205957481914493264250260371394976321398284041061172302623752686076957331502525519305442648410172").map_err(|_|()).unwrap();
    println!("{:?}", cst5);
    let cst6 = MNT6753Fr::from_str("24712415273097355830572440166960851725666256467246073930844067121998833644190597698261202506464231083426181453002881573054581642538273067658090584093919197382574963568182867199747778718416482290864563005292885839342801841984047").map_err(|_|()).unwrap();
    println!("{:?}", cst6);
    let cst7 = MNT6753Fr::from_str("41831383350893439397572098922753099785014078310970338092329705956582737118243714099018779966806954911049483359277766109754073812962220991131477763550092431612294065534783940642405964699831669104318701630719996858483591040284981").map_err(|_|()).unwrap();
    println!("{:?}", cst7);
    let cst8 = MNT6753Fr::from_str("20065062622337578290174144751190955267730418384029371768355191805973965083354713141969729090869290769497013608299386385617832071469205166343340819544108858461519910827334605671080374889965782429646303429403031692727179692744446").map_err(|_|()).unwrap();
    println!("{:?}", cst8);
    let cst9 = MNT6753Fr::from_str("12049887194960847913051924258127966963141421063901283638990518696777874499056588611550215971725724116183494854590387244617217727218633386902720962114301330389148251168913430260792209774445878920638288763835932736951523607180354").map_err(|_|()).unwrap();
    println!("{:?}", cst9);
    let cst10 = MNT6753Fr::from_str("2996252788745884520163978757891312504947356656102125240769003410403859150952753369757149443230108807306254839205978141270668860824970173210826032136263423517658096033635072659586133585287565796532036219252683454172687314591872").map_err(|_|()).unwrap();
    println!("{:?}", cst10);
    let cst11 = MNT6753Fr::from_str("26494820601318264892390169265888590073112612267084115623291357249804875490122685814299235981388230412791813691069977870921702630561473239727268951559780279711734778417633102387756053330963964210839885028600070341385484252360549").map_err(|_|()).unwrap();
    println!("{:?}", cst11);
    let cst12 = MNT6753Fr::from_str("13704703654509341311661252672477483551710457157025027646655886001558242150461703066833750902992346385717711077516573063726923733957980935606943239653021120331062230878103697738968290314202989062384700378607402676449435337558149").map_err(|_|()).unwrap();
    println!("{:?}", cst12);
    let cst13 = MNT6753Fr::from_str("19465139868431130816463122886982733801178739724909833500661152256712025568020019850217931840275735300623547555317239591734075537819718880336672120145847554656531257697773322329288152174845730118203460913344311444104093190245641").map_err(|_|()).unwrap();
    println!("{:?}", cst13);
    let cst14 = MNT6753Fr::from_str("24367710463819326632273911634692269645659248380172206031654100420210060943195035494515316451494752014713666853489053201739526347259002039378620071458063568692036498737213915537813271785979235904306992154435850712949586788510903").map_err(|_|()).unwrap();
    println!("{:?}", cst14);
    let cst15 = MNT6753Fr::from_str("14557285535695344643421782516075543477761770215278435405780607862277178368765463050343946609278574970269676078632515261133558488730072651089215659753939521093995420397225530991576502510677062653457349617332897404680148139292812").map_err(|_|()).unwrap();
    println!("{:?}", cst15);
    let cst16 = MNT6753Fr::from_str("16037354854600891769169953531496964944437495816434710124609704389510557635133638961430169712496147120582453770957705141826025133690431776716284980715439184321058047305421691642377729040657248858575423690829725207629788902563913").map_err(|_|()).unwrap();
    println!("{:?}", cst16);
    let cst17 = MNT6753Fr::from_str("1117069535358700069706938659337767222036329846720942867547582105119702469395991527900558959771417549070732058346067455922937288556121355892753614998331339207925214308222824222966051863534517011543812926240590642259425191518105").map_err(|_|()).unwrap();
    println!("{:?}", cst17);
    let cst18 = MNT6753Fr::from_str("30824471211148833986831000332296255471341075776550390259600671379725815651544772700553971933232413658194610629479275733413588850915829330939224918791472226258172884992662650525099624585943583471864905936338331694699722487851191").map_err(|_|()).unwrap();
    println!("{:?}", cst18);
    let cst19 = MNT6753Fr::from_str("1609835394291657455441339111122566491470548438575611506198101558573297384625893917601092423754560220988584562184350485967501676865507160662772259141765216184002773145251908829074038268382876813367012838323378955905297283097006").map_err(|_|()).unwrap();
    println!("{:?}", cst19);
    let cst20 = MNT6753Fr::from_str("12657231660821977719486447030756714340685373165061027495429614212520006764393434006148084446291565572519286715191708871656552872377390613536838456354375336249537940310303305204119084994699720046392886798727726258828438919165382").map_err(|_|()).unwrap();
    println!("{:?}", cst20);
    let cst21 = MNT6753Fr::from_str("6094505575950150548603195681845788944463434152545345180649596960021289343529462480069459472344868351974596950945985078378952461576423695877202049842734611101504877021625386510017407090946375703494406582549693399802108050422855").map_err(|_|()).unwrap();
    println!("{:?}", cst21);
    let cst22 = MNT6753Fr::from_str("13133667377363040587512316643038393792465975285095032263132557142553433423954973456511150197086946301467543391297598143954446019266306863531220881791090850414249811155043047777453760220821209345845461273054641288205376082753541").map_err(|_|()).unwrap();
    println!("{:?}", cst22);
    let cst23 = MNT6753Fr::from_str("12606172077944541122386936074196793807103227554419233420745128377940794723314162785038478180209122565341959697604164242066597482481563088498362439816269726848588738937222959376239274446154862939367455214768895638449996824453494").map_err(|_|()).unwrap();
    println!("{:?}", cst23);
    let cst24 = MNT6753Fr::from_str("30389195116424978697870006857174313216122323559321717021812731659103958669586382897839535957581489919253205849419850433785401530803291203590973590022043333400802933750036762020230081767678321648821158586383917875050007215940384").map_err(|_|()).unwrap();
    println!("{:?}", cst24);
    let cst25 = MNT6753Fr::from_str("40325343953506912000449083876099993177338065105352479757012222735648823875823377478377322423901425198875928671293055922818221638458721544686391352569035028044627727602152926569441150124678533569020873763219489362670499835487968").map_err(|_|()).unwrap();
    println!("{:?}", cst25);
    let cst26 = MNT6753Fr::from_str("20453578694976721425354287242352713531092366078755723467425059333253095908975357540948041379373864703633919527106273230745506466754420130982570894929837255424467605817675149827612750833799255892951954666517605993213857408668875").map_err(|_|()).unwrap();
    println!("{:?}", cst26);
    let cst27 = MNT6753Fr::from_str("5896466363480699922837915504958905014187920591615417419199357100771633583629716062328217384593343430522220837204205356283922441773869865018069875964246476782410661141757283800000150130256977907733857208823774712831127189818218").map_err(|_|()).unwrap();
    println!("{:?}", cst27);
    let cst28 = MNT6753Fr::from_str("40027595197017310469717419930330952052290625182716668238683742074119942151507256481916994635144568279469476014744353645518514008813167969542479011874911966351635914878928750149556444813765247135551298370073682815970576053773122").map_err(|_|()).unwrap();
    println!("{:?}", cst28);
    let cst29 = MNT6753Fr::from_str("28709980013818031574490474444516792617646971764778978237115164591527830630678408131371303220600167148579491465947430082007734393858233620780349276438069398011097637268171199300250904723461849799212933902077094706871635459572074").map_err(|_|()).unwrap();
    println!("{:?}", cst29);
    let cst30 = MNT6753Fr::from_str("1613108195797469890340626432498236576782118506888925067984280378752516448943140138711197508336688128659820244065610404109106281157347447090831633420701675011508770669689779072632671395876484938002258148889714201378244954642779").map_err(|_|()).unwrap();
    println!("{:?}", cst30);
    let cst31 = MNT6753Fr::from_str("2474897138570874672125971534720505367121845060732651609416891274409778147693066062916161172109922700365697841678915931718081060708753857345900572440760033144756911173007868999540623909924004051972651332774581071110938439112189").map_err(|_|()).unwrap();
    println!("{:?}", cst31);
    let cst32 = MNT6753Fr::from_str("4975279950012982949920671089032693818246406085110877430005003291337244686012802347613641194584533846956536478782413388068696808334754931265884477882105996428324054493280485225793329616333058689681651385313842857707644669263108").map_err(|_|()).unwrap();
    println!("{:?}", cst32);
    let cst33 = MNT6753Fr::from_str("1093517247701668445548965308707013395655974631634148556437450058851257136784742478271775862655251293581862316365668818804964243622470489795008857032554430795576541548221128074822109018757499262336480934008021196692653809561627").map_err(|_|()).unwrap();
    println!("{:?}", cst33);
    let cst34 = MNT6753Fr::from_str("38434962713853997396220384847591782841236466389980836190197048671539484194763664280424841327271852848223554479328983016647320744317484669715230107456265624896521894897367433409128577032875435337634720208581961854040251825791376").map_err(|_|()).unwrap();
    println!("{:?}", cst34);
    let cst35 = MNT6753Fr::from_str("32058015120007974569131756905634225661399174175760000885349847661639513114948910685511100965786031066351847931776443131799749759502155267379880229800429241593941457262025762340008916398402096085236632522654235156568539986308503").map_err(|_|()).unwrap();
    println!("{:?}", cst35);
    let cst36 = MNT6753Fr::from_str("20266563132827243230717558990935581199934613796661144736883996420445460884617586432406825949691900914858540589405543224372338376936703373540336836346974900374908878848037547118935678766334014954553617437379899917098668697957125").map_err(|_|()).unwrap();
    println!("{:?}", cst36);
    let cst37 = MNT6753Fr::from_str("545628892442224379894098556866040876962494878205466408809765703410875716268838516313326593577849779970683176637330778553206624779575343114149763665280713465512433210517707785172155995133449791944802416100959588691727944627682").map_err(|_|()).unwrap();
    println!("{:?}", cst37);
    let cst38 = MNT6753Fr::from_str("12379099697390024121343381508709199684838393660528432444897860972614583889240519686486138600135303127644622730361787586783332083256347609053926396875981970858784112224490864543818733258863739131956830535007300508302903574494111").map_err(|_|()).unwrap();
    println!("{:?}", cst38);
    let cst39 = MNT6753Fr::from_str("38196266804346956201484093463938138307290105150520127254613728862192073192996934124274713805936953830056494736357848323464737010228893280696217608625653086149715819673020527201659596167250445350683740177783562572538696034856780").map_err(|_|()).unwrap();
    println!("{:?}", cst39);
    let cst40 = MNT6753Fr::from_str("14472702873539619169783381300011345266654523712358625328804274198367885887175613543538737091182834798258429339982462191051341895364528876374080993073421212758235004769045137288684855588756843693033169118512005069794003235620856").map_err(|_|()).unwrap();
    println!("{:?}", cst40);
    let cst41 = MNT6753Fr::from_str("29512494513613526139296903713432096366002214821335635360570537733813735125391718231175577042724936248500931582734226244883113007969276876103470308675212715591859088400881972676182850086973375451534379334409339716967704265985433").map_err(|_|()).unwrap();
    println!("{:?}", cst41);
    let cst42 = MNT6753Fr::from_str("20249823293015776390076211411289153512967954015273514906009720170825373060882335353253919094560148746767177984729382290776968742510566597779008938917321037671918334550769759343840156550287833339926954142665276854775215321671335").map_err(|_|()).unwrap();
    println!("{:?}", cst42);
    let cst43 = MNT6753Fr::from_str("2454652662749141462417179776435771187592699565936899893711259592173804603269985354871976612437540407206033340241306325799349210964316769427732662084754458213517378675254706879864191112715102445588182609126740946451998667903090").map_err(|_|()).unwrap();
    println!("{:?}", cst43);
    let cst44 = MNT6753Fr::from_str("7978273401315608202062739038141907069707577538299799586346781423699636357233627801235327303513914437475066361753585532026042952527151199117588095358701768696642312145194912186116745842435323235987732554111766541934050647709141").map_err(|_|()).unwrap();
    println!("{:?}", cst44);
    let cst45 = MNT6753Fr::from_str("38385250457156624471455301259672709318074338248689654552392543051688175872618837333403192379155344212262228589811521427230622379542260186378476507877061322429975579717049235000198303767300161782887632401507408508883552901720314").map_err(|_|()).unwrap();
    println!("{:?}", cst45);
    let cst46 = MNT6753Fr::from_str("3707887441447979175630277935771277274136919694539122965313303082570420984509057198600118161006202374113050986501315388128308677483360470086062932622792523869733829778980107309928774847483419711280194617621984279765265018907030").map_err(|_|()).unwrap();
    println!("{:?}", cst46);
    let cst47 = MNT6753Fr::from_str("38543911288631359541372961410526544143903748962815403855518970948687594757015830133931547213653902344677816238921805088663935984157127876791934569396554622784360170892003992926755890448152651751610568823549158334851983343221236").map_err(|_|()).unwrap();
    println!("{:?}", cst47);
    let cst48 = MNT6753Fr::from_str("5061458209254054909582183575561347206963573941454205848354082298761298846117473526547212865583900280994346966643378609401560311720323626092904332782677052279013413232844475828888991508795613802798512810100283989503220927874508").map_err(|_|()).unwrap();
    println!("{:?}", cst48);
    let cst49 = MNT6753Fr::from_str("17267592221850382029829526256800680837243491108798216691002501619383144675533048132975677771270440458045026886263444597584126060354938954805120979161172681967107251109220455864749508015868502488666145250953871173679296644667").map_err(|_|()).unwrap();
    println!("{:?}", cst49);
    let cst50 = MNT6753Fr::from_str("19644542728926304041216405620625478190438435144818294935453277604435787969184771448086298451223405909363789277174640561831720326087658784389361077799768806674510050398877148272766402854794644741530066874294299967894367070054953").map_err(|_|()).unwrap();
    println!("{:?}", cst50);
    let cst51 = MNT6753Fr::from_str("35819862658523630779026256727418187009823269910530678756771814688836884973320224871516401074118357577073770640786510400858037794736875585343789168966586217945752205968229488720161963344389332942794176790306881547683696164281761").map_err(|_|()).unwrap();
    println!("{:?}", cst51);
    let cst52 = MNT6753Fr::from_str("20045191284788040305143220139739853960610076691319869191774213745151207495848887466771063823995062282592749561984266989606372576526901670270303016977735363329812671429085543358493370863474336851710721979775157194762137358906772").map_err(|_|()).unwrap();
    println!("{:?}", cst52);
    let cst53 = MNT6753Fr::from_str("4984076086343904783891088200583331268778734637665073884759899252205401382683427426846787965528041620017462815913090372204391251026595210575615130393930187863409308777043865038039437070016043081698235580463970059257833524786224").map_err(|_|()).unwrap();
    println!("{:?}", cst53);
    let cst54 = MNT6753Fr::from_str("36830656642983356412694092701653803733810966222879514400612409034370362467927111419148583612729886569603079734539864236694199414234616114444560139654035186701700765941284936752398549423623985214408534797103208338572836351457429").map_err(|_|()).unwrap();
    println!("{:?}", cst54);
    let cst55 = MNT6753Fr::from_str("27277234505050742414244185909529276532648724214048421585132490472209018088854581632963061554945192632576130145201389789113691024744188390681453499453804887038122394536756707045613274749499368246373774625010034743402416299298628").map_err(|_|()).unwrap();
    println!("{:?}", cst55);
    let cst56 = MNT6753Fr::from_str("8306821596190940803370138063516695864022137047753060556247971107024650691219825468032035028190955224088699862727287849156921417044549188269118240823336804714248527849228781406362404967545402728079178474364660024789766112069856").map_err(|_|()).unwrap();
    println!("{:?}", cst56);
    let cst57 = MNT6753Fr::from_str("6300877552613107632639502233329299564065501389709962741719830029950291205951557210807401739602170488037289375493012460095152553262129658283540972526056210321877515546741720709005297311062834812825062775178487251343465191443842").map_err(|_|()).unwrap();
    println!("{:?}", cst57);
    let cst58 = MNT6753Fr::from_str("12343956857188244865005855746941821084471194880071468990187021669117492650514600623954620603006365265330305439633696122471547411305074221198956938291337863296391568917345334884738520503458972484614401860576933140377666428316759").map_err(|_|()).unwrap();
    println!("{:?}", cst58);
    let cst59 = MNT6753Fr::from_str("36159807122585477860011330353087578303792438333071375962753828161179091523555469247237153823910207116986650104297898007024413925927478038276768655144576979647006400402290818822747762355905289991836264154971720585589108301936876").map_err(|_|()).unwrap();
    println!("{:?}", cst59);
    let cst60 = MNT6753Fr::from_str("35708638328248707596888395412200093460899049077997616702727192470949248411478022707626954452923911065501096224890386482780325705668914670589920132517322340156918281527476899448562327342335592343233682864731630177812266899569004").map_err(|_|()).unwrap();
    println!("{:?}", cst60);
    let cst61 = MNT6753Fr::from_str("30989444239073378630292285107908815801481785485413051072439449837383578026652318459592260555733648722887898075204628483147272357817509304910173631815089775158974884804472297911264951068356847694441792446603108184770749197425149").map_err(|_|()).unwrap();
    println!("{:?}", cst61);
    let cst62 = MNT6753Fr::from_str("22769442356429151423346758022243607546353473634431552312036594351857902577095606199044542043056630097643185333104872394725404795200928870213039138364105930026794733477942103448752989092461137200504106656447316274686404804030269").map_err(|_|()).unwrap();
    println!("{:?}", cst62);
    let cst63 = MNT6753Fr::from_str("28306851891325342422339706878493537705555836635847092442583343200864187051560094619011862024287727075041476880380742759443844966037680382252432949806995530589722520103229381564452149203404825658344293710626630953674163138729343").map_err(|_|()).unwrap();
    println!("{:?}", cst63);
    let cst64 = MNT6753Fr::from_str("12368708188793663345383229899728670473932709493158504610980790820325603723992312988578043238435327911953513366034429401578832577751918279976276115286149913651500462274741572486471588415527585104096761078105017141249149161234480").map_err(|_|()).unwrap();
    println!("{:?}", cst64);
    let cst65 = MNT6753Fr::from_str("899852016824982592401349644654541295932174861229275131745461852820086515118011108037912425618538849281246163912453275769977272828041917449048363445887915944019508098864407355068004153544278855736620321247593419639848433107949").map_err(|_|()).unwrap();
    println!("{:?}", cst65);
    let cst66 = MNT6753Fr::from_str("28546746605012626393541873055762243242706691659835948108691913765749971833359500639374235757562539207100250689528772373726070053679204585110588297142155472139903750117073419618485368202280936397403308394905388919864159364987868").map_err(|_|()).unwrap();
    println!("{:?}", cst66);
    let cst67 = MNT6753Fr::from_str("1848288163709739625618747586189891205919440699014013576954439294243644551968086910236953250433280277207862597355088985183204823410462020395203176223237506825635340843461853848403284527479301893642668716793969423112999546163614").map_err(|_|()).unwrap();
    println!("{:?}", cst67);
    let cst68 = MNT6753Fr::from_str("11969538244349590770388194074207308302995977190344058800319501470544405686633527048482980928181931634411726087423754250091989696742306955437427558108663602223401353368863699395878466351997664501997417631914912172775098367282920").map_err(|_|()).unwrap();
    println!("{:?}", cst68);
    let cst69 = MNT6753Fr::from_str("17761189259573432096709974457474706725985913250933519212783991294984046883190182054118189509895649052598898016790619240332150841899324714924041835071908000885133475455122128121583201488901178523851388153034982420487543611851524").map_err(|_|()).unwrap();
    println!("{:?}", cst69);
    let cst70 = MNT6753Fr::from_str("24931257694648558025260978528659165035539072658818884354661511816279431413920486319329086699622820458764965318563871943697922364439863295798505825698290617565902802539933322084437000636132032629487839586078415243301635693520743").map_err(|_|()).unwrap();
    println!("{:?}", cst70);
    let cst71 = MNT6753Fr::from_str("4703674817641278056969042447226555419255585401195707518952021714014960175577343973007115290839154685316660172030229711663264710083334885899229466453955730759563487829051966522389894622565546292642434605317495429214209825045571").map_err(|_|()).unwrap();
    println!("{:?}", cst71);
    let cst72 = MNT6753Fr::from_str("28117856711185763742727353211532543098430567503738222778190476338765072822938789277575959251366731065136314006634007922547289415629090512661120168944286788081425569990318139431628625056749923722509423289747769220754020647494882").map_err(|_|()).unwrap();
    println!("{:?}", cst72);
    let cst73 = MNT6753Fr::from_str("14266160474499618689926047159829097857033955849523690023013220739770223501980719064677135656366609284189232427120970929928810102580348440902572207083622415532782758347518861460902437211925457615002789016058409665720362542706407").map_err(|_|()).unwrap();
    println!("{:?}", cst73);
    let cst74 = MNT6753Fr::from_str("29485117419544632947700819752341724863188706269557363616811184565158815212744995058306654302478449229806853263725717252138741501594626336593192970342913462678750413251322408513408420482797356527687126970800722148134672266111294").map_err(|_|()).unwrap();
    println!("{:?}", cst74);
    let cst75 = MNT6753Fr::from_str("4774098672098118424250512906759092333848275003834168016399796954159370299477656132005567623116352953481173071179646540477558004584440739632408954841659504866132310474225516992157478343313764842906225061804993931282703698294397").map_err(|_|()).unwrap();
    println!("{:?}", cst75);
    let cst76 = MNT6753Fr::from_str("945186695736104272935684362145945071997434161190521153350857017937270744295308469453239521145584112171193923290673110056711091725809153488755122788403435661376304377120953678361463993973826505832838665540936426907300548524166").map_err(|_|()).unwrap();
    println!("{:?}", cst76);
    let cst77 = MNT6753Fr::from_str("34928044303668445544266903544554086449059410752255959921494393821877641804292675692389880584444411395394992958167165937863293226254752439232100211093033437433592127646045843717209370351349600536307179437318658753968736611544103").map_err(|_|()).unwrap();
    println!("{:?}", cst77);
    let cst78 = MNT6753Fr::from_str("26857137827306455456619764868713888544748788910106160648426836567640484208661162237766702396174961675261196375485533202712604740706409591043557760055309165928879130437941542221593109632853959309078856165957440566828663675808130").map_err(|_|()).unwrap();
    println!("{:?}", cst78);
    let cst79 = MNT6753Fr::from_str("9991428332543937322077028770224407002014414777265865405182879769507413455567286519026724928224064327807571618305092325436107136159218759087065362989719265262561332322319521632226720240442783355787804157590023806319527026894136").map_err(|_|()).unwrap();
    println!("{:?}", cst79);
    let cst80 = MNT6753Fr::from_str("28414352916502118982671879553665380540460534215033514182490339804924771531524570855580336527434014380713440076202387874118763969488811654405557468607270032910231338296281409352846688611562775450707901195099896481872572379701570").map_err(|_|()).unwrap();
    println!("{:?}", cst80);
    let cst81 = MNT6753Fr::from_str("3940404902152415759962934450443433496824814501958597562899836433451975797840349339016191096024469322736926308734651719139779818929187501002059275171917552144503194212090451398641456619869116670107766272065616990633496874266301").map_err(|_|()).unwrap();
    println!("{:?}", cst81);
    let cst82 = MNT6753Fr::from_str("4906506626882533879316665426679942231767585287372443208981085345737635446549877115485955342683947733960393253769490522023526929389260911276733898784080374956990399332443707186383023538793562557555986402545637311130198352732614").map_err(|_|()).unwrap();
    println!("{:?}", cst82);
    let cst83 = MNT6753Fr::from_str("9238821282617944019693346469979727455399572175212643858759466879749790372587262037263768870556587144064570215436927982531968089511731993234863789768512874484683137005944747478057534392703363055618780503752186654106308366842276").map_err(|_|()).unwrap();
    println!("{:?}", cst83);
    let cst84 = MNT6753Fr::from_str("8429192231075353742830730588272644861609406447552544899891846119452101864848678013115064098100960946808917108238372388533436165703570542917271609618996802913584113894895692893755466190553885457014847643215511751596712177166378").map_err(|_|()).unwrap();
    println!("{:?}", cst84);
    let cst85 = MNT6753Fr::from_str("3839479080535509084907405258095282747725212505506300605623459595980669738240445568855487421417386486274169662480085716668079832786435021917807053189646291078693719080866575733840390323821459240797966912726393681899017825335058").map_err(|_|()).unwrap();
    println!("{:?}", cst85);
    let cst86 = MNT6753Fr::from_str("1535534340347009385152861317281841533331355020062326566835354968925883770056534626448717855169076300035974337587019894768573248246885737870834178406041948688382565386050656595268448472547456141670352036922532344977200149230084").map_err(|_|()).unwrap();
    println!("{:?}", cst86);
    let cst87 = MNT6753Fr::from_str("14549491564173852951016007803830729180927991637774003757457635629327451572174980045401621327651364605463047835851395787685841042053900851953099953033755008352607979849001411512358532949085805375463112958390777767878044664724866").map_err(|_|()).unwrap();
    println!("{:?}", cst87);
    let cst88 = MNT6753Fr::from_str("2782025260125381439941093469571008710556616691538196536872814788286938108513961815337087746565106337439210871976411670128454075162091760859259145015630291516069605353373358279547247042143444803839908184039037241116134653336903").map_err(|_|()).unwrap();
    println!("{:?}", cst88);
    let cst89 = MNT6753Fr::from_str("13499565986493658952151293998232880083932930761269440868524330293017095727135115856269762214071631464876473030722401518593345821145280498682862357730337051544587726214268509650776332695844092603983680698118038346827168236024897").map_err(|_|()).unwrap();
    println!("{:?}", cst89);
    let cst90 = MNT6753Fr::from_str("594703249855816929566454826919339136820153519724134481076231238823475868222742354595785775333530465521801860542401531975883358240903763077468663135885150615919372464047419133822346097101765431136136845631715109722108001838827").map_err(|_|()).unwrap();
    println!("{:?}", cst90);
    let cst91 = MNT6753Fr::from_str("22043884936671239278262281612785423939533073097934967576515001027571149716688504601996998170714431819324862348818024861341162198783329607168896584546184211891858120429117317080271800465911081725959279427861111913034313579186237").map_err(|_|()).unwrap();
    println!("{:?}", cst91);
    let cst92 = MNT6753Fr::from_str("36123337769816951462938108288709120431803389688301573709813917467007428882121537289174429909177370685630739117327064578713858010008766552764536393665233615927728521277382800808858760386573993564644033734937844682893831941964740").map_err(|_|()).unwrap();
    println!("{:?}", cst92);
    let cst93 = MNT6753Fr::from_str("314459599820654163268813564507565432918906519581439691235726237598379953363651664860888084163941177807825101629894803766848048058727812443600140006864624433522573414702405551462769060749464239499148467185283170282981563478777").map_err(|_|()).unwrap();
    println!("{:?}", cst93);
    let cst94 = MNT6753Fr::from_str("17933697803555053672819344466340794322755118461710173159121525596263513292356238593924405979540832843999690447246461868486939278643051373860277266403992095540840498186302197438572155510706902499012241571518099894361344560471878").map_err(|_|()).unwrap();
    println!("{:?}", cst94);
    let cst95 = MNT6753Fr::from_str("18831463694135178396861660075371186024122323879957255319396818677715160749005887421271450219633273851226822704506899498711827372126479494427582575097047326297600315926804864481184776387805490193302528893373904591945027256408485").map_err(|_|()).unwrap();
    println!("{:?}", cst95);
    let cst96 = MNT6753Fr::from_str("10545508765744707184494235794386641885227040572780315302520052267097267111080066107363148988249759630504014362629247608329626739058699645566640103510097237413345961593466380166966239947041331906307182126811920471924953142563625").map_err(|_|()).unwrap();
    println!("{:?}", cst96);
    let cst97 = MNT6753Fr::from_str("22398493565117977616544590036623562326305825243851269578665929634960629878418766291085602678780706492696677985791226616525733984096641647894599584352794716766929128230568643198501685796799548845347102101422297521568029258541463").map_err(|_|()).unwrap();
    println!("{:?}", cst97);
    let cst98 = MNT6753Fr::from_str("28328649377424280881445828098677341512188799782564210065813350310359003270697452487750421847437707452772791383681529836106484695071217054287032746741897344765928614707812438526477855631909007001352958223977531669376337172488454").map_err(|_|()).unwrap();
    println!("{:?}", cst98);
    let cst99 = MNT6753Fr::from_str("39725284316261598526363976512435751788982499376486924433052287878671785802100897624095080910930661516719464139569267464223797346729741672878683169229468453184196372639151094275814466806870129575324095591341061629833086298688961").map_err(|_|()).unwrap();
    println!("{:?}", cst99);
    let cst100 = MNT6753Fr::from_str("26777277528792111635411935778413042629396278019013854410714572235650494962807116283849102909714115169834067595256552272661658692772379331837479469146614644392101367072627720379555797187812234917615434789311837411388955705715730").map_err(|_|()).unwrap();
    println!("{:?}", cst100);
    let cst101 = MNT6753Fr::from_str("30237173149984647254137196596335974838921990964429591901846908823053036233765568109765788057368895691146432407644335782227476565512663467955803930511956355582547826660860097300843945332293521280553884205929991701736026638685348").map_err(|_|()).unwrap();
    println!("{:?}", cst101);
    let cst102 = MNT6753Fr::from_str("37307767259310458608917799526349555136165327720862467551929780234794465463766063559249146019720427871947089638384261425158436578695303288173430833392632871311976480575231236482089253463409793524934767306909926016014284210690646").map_err(|_|()).unwrap();
    println!("{:?}", cst102);
    let cst103 = MNT6753Fr::from_str("15634894300526691977575151562564730588926457665331409996056505091276028069129048115021428469224461710563689104925976707545937226028106025551920862370096757846516538855237345312751376167072053673718832290224769360287996690510509").map_err(|_|()).unwrap();
    println!("{:?}", cst103);
    let cst104 = MNT6753Fr::from_str("19619125455570965793247430974914063708811703025545509576268645235663841134048893197978927266483151519048043187209785906579961226877339563525597896048183428678763343132930467886974804228266617302215414860236172832764669865968223").map_err(|_|()).unwrap();
    println!("{:?}", cst104);
    let cst105 = MNT6753Fr::from_str("19751168891722819309332086723280062954934725636893632863846659046369613357405938718771852738007079791343351089621099879941422124886735985802490124808604897357159981838415923014382189702633722926147212519477530484535116610704970").map_err(|_|()).unwrap();
    println!("{:?}", cst105);
    let cst106 = MNT6753Fr::from_str("22061382094324366275080877669780503202644086912022036652375794192111025820318803057896620639840101046328815197705807335391083608833229664101336500641062683040167001024503432868641772338892928495176202309285368907289758510193168").map_err(|_|()).unwrap();
    println!("{:?}", cst106);
    let cst107 = MNT6753Fr::from_str("21603626763075121361603573563429249953170482296834055218582305000394490707617210469477553294906355533299229248518570424471183486984599057068496752796224994887103278970306591771309218756148211774605876747674125116348957483117212").map_err(|_|()).unwrap();
    println!("{:?}", cst107);
    let cst108 = MNT6753Fr::from_str("30413079092888523518047928224563966573619119428790915702237527676484518176372383819183878104909990135899932188175199593050603256251099848850196696338466167148143473941545772298630488294788771474760659187418055770004602082592528").map_err(|_|()).unwrap();
    println!("{:?}", cst108);
    let cst109 = MNT6753Fr::from_str("29871290633605509826668057098139444299518585706510194143740888580370950728287648606746990724526315708995320225554883719813133782191904907442497892218209437736341749545502871195971501178681566899829050481687266831355239686506730").map_err(|_|()).unwrap();
    println!("{:?}", cst109);
    let cst110 = MNT6753Fr::from_str("33230636723046882948430886655783209359586118872902071393795009750947834687441614824336141835226046908377883800420854827706688664630963762385415864093493519890211697608706559555303037312710119078243163061380856694936588168624775").map_err(|_|()).unwrap();
    println!("{:?}", cst110);
    let cst111 = MNT6753Fr::from_str("11422238416874454075282616342813997063959627473927508263066910655190598399880356998591871896710922007945143946008580064363305899284748176990647889348568856179680955293038075504796430101488592939762295878739600805840277749516677").map_err(|_|()).unwrap();
    println!("{:?}", cst111);
    let cst112 = MNT6753Fr::from_str("20941530910655587320118201487607027797069444673732273301339836529249718384263382060669896459880368283117348150800468588535569132372069684465104631633149698695021445063578330800727962851199686477572409153781430707270995934113300").map_err(|_|()).unwrap();
    println!("{:?}", cst112);
    let cst113 = MNT6753Fr::from_str("16754962502499993743803208751472573392999273520497006335668683378873863539121603928591887214145216927483856270998291312045869331709377016303114781535385203909219084580769490482260200346851077977056545577720227668257783940162102").map_err(|_|()).unwrap();
    println!("{:?}", cst113);
    let cst114 = MNT6753Fr::from_str("8020956595527435968788548596790354237532124627363242836229087697459652004059175036505507413724490800318688924090688977897809824964546585748284980398172336340842351861296055498898361725425995038826900688558168829287042717379215").map_err(|_|()).unwrap();
    println!("{:?}", cst114);
    let cst115 = MNT6753Fr::from_str("20476363823267437370238854401082906074815310479376993903303139247681355787834677115410458000853606071528498283154394076105610252162262880882638171721366957433627549302601455797917930649660803470752435808315126296403930491991410").map_err(|_|()).unwrap();
    println!("{:?}", cst115);
    let cst116 = MNT6753Fr::from_str("11786907598606913730832061227784004708715621242261935785210292538916530267384270126860575119541636136508137778125857997523809308861196165747565446854512341433121899683465913258641340433960068473072680069325116995546307079044397").map_err(|_|()).unwrap();
    println!("{:?}", cst116);
    let cst117 = MNT6753Fr::from_str("26737292753837382996954766319240240977376261025143063830338220382886374620051024054526435844095061518365447495403849619020823108282012020585060091453038132569739166071265466927776509495937177760480875416894446005333400583661232").map_err(|_|()).unwrap();
    println!("{:?}", cst117);
    let cst118 = MNT6753Fr::from_str("32761413148006957376261418567736339005160012072856669704714512513136823413467545745762716458491587140511188368465475378789925391583234425747875685328069426321514962871266765778325999296362165929429965130443184484017499302692582").map_err(|_|()).unwrap();
    println!("{:?}", cst118);
    let cst119 = MNT6753Fr::from_str("39661034793373639645959664837031798805562980447141335838537621951615629280854682087097808394872997520280464166895720376787586280112860070779736592845468226198085619006944849210730079530258181416675557741811843556003562520685794").map_err(|_|()).unwrap();
    println!("{:?}", cst119);
    let cst120 = MNT6753Fr::from_str("7679530021775105404665637963701898615864938832771719032365598655632398537308455876191804835669484144146200555838016935549331210519446337510889933096478575566017293567269317386874582867911529460532345919199036696189929672894026").map_err(|_|()).unwrap();
    println!("{:?}", cst120);
    let cst121 = MNT6753Fr::from_str("37478100332948585091947912948234100704669809091735055191328302195567529735162293345293873077416340865588832645538390149516872287189056745632986350115673867409463971739811846003812754601603250571248220532054324587097726386260318").map_err(|_|()).unwrap();
    println!("{:?}", cst121);
    let cst122 = MNT6753Fr::from_str("11405408830917363355190365619128767680701552535954157703913301946176835007928909061804205097088448530162621943943033203944569913818253174322581616554096095343204530227115705181876614623215328949263729090132449535073183535056007").map_err(|_|()).unwrap();
    println!("{:?}", cst122);
    let cst123 = MNT6753Fr::from_str("33449802541662729787507533197730558412671847031453826134143899403287528494032289240503499673130875011466037498921031478813003185629308032102679163735315038554543080451117675479937050254124148067620925175853567256601065387048160").map_err(|_|()).unwrap();
    println!("{:?}", cst123);
    let cst124 = MNT6753Fr::from_str("1095318261386050169865238373488777596359789539216627382549258030863465172643863601274203259620187574990704447182289257096761362427813890262106812430353255715454369408456561290186365041233739168499473119356448508590565211451503").map_err(|_|()).unwrap();
    println!("{:?}", cst124);
    let cst125 = MNT6753Fr::from_str("7290732200123674575580485140573802918961192233095047412955496709463569245011819892492412069253458567978926999039366183529766786716229154171559363589323107821296117502028583643682859421427834269001177318657978623891916981129269").map_err(|_|()).unwrap();
    println!("{:?}", cst125);
    let cst126 = MNT6753Fr::from_str("16697629005907161666399057719772179563480930572623597751777449500288981610815118412263271861407775242330098679826812420420449008608277523840087894577078867299117528501649237351324440036603572207484030534347502259798840868600065").map_err(|_|()).unwrap();
    println!("{:?}", cst126);
    let cst127 = MNT6753Fr::from_str("15295272898674892783598391544769981185707378812202272669393459710522561272373800289311284980080534582440851095873865892055442997183151860138669974478710219633881282918115765075854963287181901464047288888170012304401126823923509").map_err(|_|()).unwrap();
    println!("{:?}", cst127);
    let cst128 = MNT6753Fr::from_str("39836819991178937201057903184059224319138688556531926473533842512553103485840217104964631976584783286869935150056280364109658734820692767927592273991455796391432822221979512189226715629163849352470477240099951264156512979661091").map_err(|_|()).unwrap();
    println!("{:?}", cst128);
    let cst129 = MNT6753Fr::from_str("12966006174647134741264705880683880233136009816042617016143172072859807907149574377035960944866771687873093794861489435112659148621235595424625997991324282788166253125091199651509868246160946868880273168265405535408687969615035").map_err(|_|()).unwrap();
    println!("{:?}", cst129);
    let cst130 = MNT6753Fr::from_str("40470343851918608637872993348642830103983898785311951076961123487360514627093218120393280093623616494971972140547044246579542205218444031875469520874177708992253559892429185426193880173119133047339268629469041411937220069154814").map_err(|_|()).unwrap();
    println!("{:?}", cst130);
    let cst131 = MNT6753Fr::from_str("19897174597482036277304433206055894036389321374549433031322670846656933154922174061624239193814051575196835453830336379934988420770772275683492244035626137425645513832597277879361374880068663783355721651895620599249132956092588").map_err(|_|()).unwrap();
    println!("{:?}", cst131);
    let cst132 = MNT6753Fr::from_str("8601190579047454540750742554962308480965931038675880634812340291113063813110558052921554724070382425966667867737880686316238261082546751774566626898690514715652448861460162191812644513629135304618313063379991418923071982303772").map_err(|_|()).unwrap();
    println!("{:?}", cst132);
    let cst133 = MNT6753Fr::from_str("40277905809339408264015005344171372690774605884249118789614801989403203785224378107918446010244308702780991844124923803029013156521799517125609714088537957825358538710647281771608024317494452288970845430181957403342799969936523").map_err(|_|()).unwrap();
    println!("{:?}", cst133);
    let cst134 = MNT6753Fr::from_str("28830145523563918713457044075279228420322520232374102088362189156923911145887952010825572011386509495886354957380587326813451883166935880263520094035467187853536610461960975996221351067699699733720457275040893058043073002186631").map_err(|_|()).unwrap();
    println!("{:?}", cst134);
    let cst135 = MNT6753Fr::from_str("9897709012004954633430636864859804676955330874263545172873805979583407218481175773525769361922541721066121351319942566868080435627037954777909788976258449810401458783536096029098414444014393865020141894837565890191366975352499").map_err(|_|()).unwrap();
    println!("{:?}", cst135);
    let cst136 = MNT6753Fr::from_str("14967162809307649766577117986852087721432635328799320204863886576192073442827408863518674067978616037271231903941622885053848748206780541109882675448738051872896675039519720292130007323529192592436724380887370592938693610920232").map_err(|_|()).unwrap();
    println!("{:?}", cst136);
    let cst137 = MNT6753Fr::from_str("22241454621784953660343927582482154802931302947792200909850267563074868008067197753506258160502319007010260475957223045669954436028490283714718481382271136250227584007626840045765048162246075713424326739852198548198128795200378").map_err(|_|()).unwrap();
    println!("{:?}", cst137);
    let cst138 = MNT6753Fr::from_str("12540843276718056085698223381343156918343245248854251084956384760589812274383655648214166124345476117385707228134474721157907382985195009820362188474607118255654964484902824296578099838778526840388320525176398486333922810507831").map_err(|_|()).unwrap();
    println!("{:?}", cst138);
    let cst139 = MNT6753Fr::from_str("32566149955792884064614190388771835294173137245407795950986595506896986862496171980131283469081276630140539847539264143986547108424129741272432969186562263430783814214865709393422463103945445285601841563354447395661034417397059").map_err(|_|()).unwrap();
    println!("{:?}", cst139);
    let cst140 = MNT6753Fr::from_str("40751902366085299050439992977196483015554715950556965942668056352257291964012743022117545028310677346439236091620163368117843105501974786986110240466196747903076260422602043200096913007685296255101572184853785977896582257388118").map_err(|_|()).unwrap();
    println!("{:?}", cst140);
    let cst141 = MNT6753Fr::from_str("31347304480512538436674185507412274992000641867177831972216002340086850332482309624610695915648716966412443740558153346169052819881844822110374846681120498942204054642664837972793567363562900502438118366794319392372894085242013").map_err(|_|()).unwrap();
    println!("{:?}", cst141);
    let cst142 = MNT6753Fr::from_str("2900947603642700594498449884972229887280213756057556195778823189492661314473980997997385993201345559752850468260700758550381632947819937272322386438963789289268763850631262455186940780212879187278742052080535635287443680111726").map_err(|_|()).unwrap();
    println!("{:?}", cst142);
    let cst143 = MNT6753Fr::from_str("14170424648634730071510704674116577574143405890750257077296612662653125989319243158370942086786333623541803396037795336740280251970574582404050211078795080996347919160389043304898278944623007966384405349902873196044812697600085").map_err(|_|()).unwrap();
    println!("{:?}", cst143);
    let cst144 = MNT6753Fr::from_str("23622870109669537802557498346904054033009496755176322247268413285802938419877860606570802294147017116948942360130677145493188836821636438837895692263940211475297423300428825753934784747354075566255484486751197711354479848623586").map_err(|_|()).unwrap();
    println!("{:?}", cst144);
    let cst145 = MNT6753Fr::from_str("25153013283415295998285628320110148360042731110912986849431517914930683638083775074986233584084557018513661633131929930204247713824123021628383240462713994444164019279776705170815153962272503861421700700809789171374323410370097").map_err(|_|()).unwrap();
    println!("{:?}", cst145);
    let cst146 = MNT6753Fr::from_str("1246783888902532504699196777640189802174524816118694854451187159303582011058042226996360234733148964046125813904775984891536126954960702896279314993719283664238898735919320457313228898944382370861948367738794077284742922915929").map_err(|_|()).unwrap();
    println!("{:?}", cst146);
    let cst147 = MNT6753Fr::from_str("15651333107391299893427668307885316465197650178680546800327966080037952708631761798484846021471390663273295869898244830832124926982876720592148634969614098583796715911938033561452913671063193491194411728883083231605476970509849").map_err(|_|()).unwrap();
    println!("{:?}", cst147);
    let cst148 = MNT6753Fr::from_str("6676240877579730140554224026173454333118584022673175685502387332694196881967900084628085566505111816904155420396337639900853161518230639765971279702858147671203287043093244560081271196928079885539685404708714087161233252556402").map_err(|_|()).unwrap();
    println!("{:?}", cst148);
    let cst149 = MNT6753Fr::from_str("9039854326835654736645439619038126921792058211578165821199703162498765427805846095492313980122099236272428337206707092441393351579808845633090836362148914163128079018564200238562638760511291961195582867335831527952402895149414").map_err(|_|()).unwrap();
    println!("{:?}", cst149);
    let cst150 = MNT6753Fr::from_str("25860550081644513684930815092232224894910303344999853602069551012908628844421846541686150226758722086943069480445908106643148096224411624213939344871215867935202385410021618491300306527823968396254019859403113704897332042899924").map_err(|_|()).unwrap();
    println!("{:?}", cst150);
    let cst151 = MNT6753Fr::from_str("494328143778930057421602150932413215178151850604881392989222955919346405664309528551608966721372473532014681211798182582589553613773600840856111507312014923877160649523004621570162474713351592610092890020286015829392081942245").map_err(|_|()).unwrap();
    println!("{:?}", cst151);
    let cst152 = MNT6753Fr::from_str("3235778590535504972107259490979344071059388932772031113716035727056169634921530565919490559690098388274233967437483624528928163712395010779041674450043804540553721661696206467159991486485697893468999724225108361227205984651464").map_err(|_|()).unwrap();
    println!("{:?}", cst152);
    let cst153 = MNT6753Fr::from_str("11377110767656041450484385552849916292611988996377928880110791952213194369325849710942809423576943547005882157330734043391228080129232825152215524755832218188254224842851475567897118181580887364549494627375518226093685205889312").map_err(|_|()).unwrap();
    println!("{:?}", cst153);
    let cst154 = MNT6753Fr::from_str("700680624354942661448520479011598021934917813160868717790338196983723205442851309763935194616480948120589253191331502337814817244457602994367921597481676530827002274388158450201753651647783817758823999295059213917355776298832").map_err(|_|()).unwrap();
    println!("{:?}", cst154);
    let cst155 = MNT6753Fr::from_str("2528441836697311933303532719812384614261867957773623151597619436650333002544666161375858976740624360669500467867479918145678334430995727676074386649782535333320873892611301188584474421075979496046307928673080112625756694866873").map_err(|_|()).unwrap();
    println!("{:?}", cst155);
    let cst156 = MNT6753Fr::from_str("9923533064948599717284062984631685124152765931475925140859676959360177875884235222279123523218564239654723035919430358776191520722153261566359180058337422282506434247603209519281370496506584158105354163586899319075866239503851").map_err(|_|()).unwrap();
    println!("{:?}", cst156);
    let cst157 = MNT6753Fr::from_str("15729533690666701318044801644353139034809257570423468443516227054643332635322053517357945715923722680416504365976210035585267162431074876639598532949244147442121304120310504309864482328902317256279742905086367999842877536650600").map_err(|_|()).unwrap();
    println!("{:?}", cst157);
    let cst158 = MNT6753Fr::from_str("16940861908688340122022680060757582922006041070752424520038081591502073384269000164578058180764582003978318997100273018477343865033627663625209466145032152873546918672248631375890548460675222269801063722162513879113884060568964").map_err(|_|()).unwrap();
    println!("{:?}", cst158);
    let cst159 = MNT6753Fr::from_str("28002289862204203749728944996305386166260699580762090433100260219315076609808889189676479392061698994854899026960360829921236541785380093626092527614027846299190610094404581020216684120489648041439464240066830926645654327655124").map_err(|_|()).unwrap();
    println!("{:?}", cst159);
    let cst160 = MNT6753Fr::from_str("5744958733253570345940540409689367621166518555967524424251526365109182153473295880347509831215793386133221883493713578969084933822214986355304663895941443683569591450640633838740822314634826091584646281372322355731738321055487").map_err(|_|()).unwrap();
    println!("{:?}", cst160);
    let cst161 = MNT6753Fr::from_str("41238887431646673284546121770819656617143077145324097476177014571003274186232847184635162833801953717023231930014567986249317162739881513825356374770645867063081115632322378378293302043994308803482620607959088324481631708791034").map_err(|_|()).unwrap();
    println!("{:?}", cst161);
    let cst162 = MNT6753Fr::from_str("27114250966389824775250906063056908480238498255508041031177203055384838474951895157474498928259614902951285888703685513848396852062961031546984484291214212148272928293251349463259178406051707375390754219458746431486719687174077").map_err(|_|()).unwrap();
    println!("{:?}", cst162);
    let cst163 = MNT6753Fr::from_str("16353602833643514339890002158442322352697327625186162255384840434769042629667764932705586986968165978362097022765967153525188559390956703261851433617538778317209363286316140074575746914641847647540139945043931464180931765454909").map_err(|_|()).unwrap();
    println!("{:?}", cst163);
    let cst164 = MNT6753Fr::from_str("24156213702877047079873195909341034919987232454697918089362995643373786356204034589339277338634264195288724013013096197221735269489396529296152688081320276878434817794901909305415993495447439826045846820335958290703230386224735").map_err(|_|()).unwrap();
    println!("{:?}", cst164);
    let cst165 = MNT6753Fr::from_str("18679740323900115017901750367084019581091518034488871097385771597981983716323638704331232522479448429806958570135970942010897296623519647099697884070432388476474344375477361133230415011395716907003913955508333299200707649479579").map_err(|_|()).unwrap();
    println!("{:?}", cst165);
    let cst166 = MNT6753Fr::from_str("13661894154609996008171599020232154763719358500309049700192431830329019769695294436766409788749464028225978679794192961411165662175255441662722750069576065708080878897754348024977841728031050256726997429305386993648574385115999").map_err(|_|()).unwrap();
    println!("{:?}", cst166);
    let cst167 = MNT6753Fr::from_str("30460214733618688147690488292305890169284829311874507753640509595165027217624547205345914900711643375248471785550800589891762615289094230576230063025812487550685582610392744925113421129871143534761683361230591184255517960596783").map_err(|_|()).unwrap();
    println!("{:?}", cst167);
    let cst168 = MNT6753Fr::from_str("8042985651735600920346024480763321246630321612650408258510217330100064391841764494095562814647282440773073099032619105632664247870702182306100490724615639701524221947018616019042050868172853450991559621927386676291103367863202").map_err(|_|()).unwrap();
    println!("{:?}", cst168);
    let cst169 = MNT6753Fr::from_str("6399119887194979617099883752187637862707013359000820540604799604283349211432405700999092339462561096846219637918139285608351890986683997734202586380739358497854016708033983656378882489298571013914171523401776470274707143914309").map_err(|_|()).unwrap();
    println!("{:?}", cst169);
    let cst170 = MNT6753Fr::from_str("2111377000771788159765224311615409190365383337440290214682905619502871739589447719690814449095405683009309289510151929472635173084271737504381868182608788054831674415449536914512709108800860534299562033302762163143175376927444").map_err(|_|()).unwrap();
    println!("{:?}", cst170);
    let cst171 = MNT6753Fr::from_str("3849615049521585259751989612451993909779410232614433409510495561228659566643021081506916054595084899435376752127436287932545946449830455627280657608545976976500936922797593284700081322506646829386733088655965396311505997880536").map_err(|_|()).unwrap();
    println!("{:?}", cst171);
    let cst172 = MNT6753Fr::from_str("39776958856000429014795528318329445528249729464860910533936191669597829630526367494241578708899056282819745101682344996047092444234828334205904428719712938765833443847671378368229483717896682478004582944121501059809308012461319").map_err(|_|()).unwrap();
    println!("{:?}", cst172);
    let cst173 = MNT6753Fr::from_str("33931257637018651112656685050909161091024939951384936700847602417234096876541155936154414613916523123625037835904738968620857985755933745267160419593488560231818636911943176875697432938219818273863699243655963355908158196385784").map_err(|_|()).unwrap();
    println!("{:?}", cst173);
    let cst174 = MNT6753Fr::from_str("22020090588757341015759133101185511720012666344416363124588324379268272514602608451163830372285225504742450184886867403292593164062971287583802590977084441228493593425304426799843649857185070094717754779490358216476043732352061").map_err(|_|()).unwrap();
    println!("{:?}", cst174);
    let cst175 = MNT6753Fr::from_str("32528553497482906820281005781571139643676601603263273737550230897699699795831598595493290137035217367786022202601717866276364793352037906429546708090150191410742906611701678006694319544885875113996791746703851050839400321782849").map_err(|_|()).unwrap();
    println!("{:?}", cst175);
    let cst176 = MNT6753Fr::from_str("40002742725930687231402316657846133832113729865938690106047171237220558139294618784722574659460346924672842233166458272510871438958677343499021487701706919326433742741665906173290152828666913920695815585117772834567545804174044").map_err(|_|()).unwrap();
    println!("{:?}", cst176);
    let cst177 = MNT6753Fr::from_str("34163097247929430782115010857817342341772434798843661033298638194369458299578861874162787239947752830181975300834801265031172235171086884859763739555603628165400580525260192951943268200047579706893013930835679393139487623530338").map_err(|_|()).unwrap();
    println!("{:?}", cst177);
    let cst178 = MNT6753Fr::from_str("1254701933139960379045832219155029356849608523715096411410317147935847797016915143570982883461133931858754577427608061679723061510682459756496707380035594891828182451478175552495000221371107771087785133200981360950321579997165").map_err(|_|()).unwrap();
    println!("{:?}", cst178);
    let cst179 = MNT6753Fr::from_str("28400688517445120380720293548814169530390484274754947589491284723138661552281999464923882544435658745500321360943327284398363525680558428945232764565116640630682905395663589178266038889451795480052307642200814136968422052651525").map_err(|_|()).unwrap();
    println!("{:?}", cst179);
    let cst180 = MNT6753Fr::from_str("22677735830821477086567443262827591337363166947301372946469998414365865578716557587011663873019717302004471086059825009338215073108836612136529408480990820901421353185037451835899381056736893871709350950451120004868511889805983").map_err(|_|()).unwrap();
    println!("{:?}", cst180);
    let cst181 = MNT6753Fr::from_str("1611303550239613880711453221557336141369301367844123006790398222401825261394874184664750706473894631028034196762634665137302285840335727190473163754507585191667786983537286219330405679456105662278546612586207126198381212415331").map_err(|_|()).unwrap();
    println!("{:?}", cst181);
    let cst182 = MNT6753Fr::from_str("3763084791603635340092521963445723880466624005587080142795146590808819503121863534850944608669684598808044385074795950905409119764087869186805731871758376030884510851485285516774508938483684047519385200255320651676145573446594").map_err(|_|()).unwrap();
    println!("{:?}", cst182);
    let cst183 = MNT6753Fr::from_str("9840966234549069477468591979230098944789809973613701515400766791081677635643515307296154440623613140826230754009414099030358778137647495635154178143265302726366799767334792212177735000856126198515684941850340683990484875918719").map_err(|_|()).unwrap();
    println!("{:?}", cst183);
    let cst184 = MNT6753Fr::from_str("11274072893230139877854502373353236073126689372051642389045008490220271669743889285321237274278308778585149048772319076106907652431161307390766303413491846291430938625415262736963860280238482093345121219725891369811191921216080").map_err(|_|()).unwrap();
    println!("{:?}", cst184);
    let cst185 = MNT6753Fr::from_str("13195761844546817953171964154541820893300217530144868445877738183128462800433455367321674306360974695410505816681730211484042655243868975174936524266715534112547119741672268181462919104653202510838633081878757687682818544702616").map_err(|_|()).unwrap();
    println!("{:?}", cst185);
    let cst186 = MNT6753Fr::from_str("3230094832816483111092423171854523976033610832412118113044005431132845017430780689791804289821533301908155032729121826142125346796990677744482787084791105325854704815791104633240853104837105240053462454302643223591255629059510").map_err(|_|()).unwrap();
    println!("{:?}", cst186);
    let cst187 = MNT6753Fr::from_str("4075061692725661152863443614503512066624565769690805995295092119728159298404781927993019273760555692984417085610311948905336898684620066298418463048581383081889752060219263916374676265673471500697395950151918150833627154167514").map_err(|_|()).unwrap();
    println!("{:?}", cst187);
    let cst188 = MNT6753Fr::from_str("5928150977978840624637886446454462501782086050562965656954497515497414716663213964668909356219036336148813538192198639567225535800216782520829640876150574457794290923763126253524401295840425751063647217024834287366412899803043").map_err(|_|()).unwrap();
    println!("{:?}", cst188);
    let cst189 = MNT6753Fr::from_str("8530897196563881359561286781709846522132942669492091237050708096234841417177250487963356206245591521534207540966964304701254716499287273227047019460863780739362356099132888931417986987872019230816891016804473773185194010902362").map_err(|_|()).unwrap();
    println!("{:?}", cst189);
    let cst190 = MNT6753Fr::from_str("87595501012972675516388119350742812059360964650864364311699181139226231018339173250602416977219685117850837037160112084714347190631179142227729838938213591659796571154191753178772512217789967542973258576497603906208821788557").map_err(|_|()).unwrap();
    println!("{:?}", cst190);
    let cst191 = MNT6753Fr::from_str("4814928455002236779141205005373116585288833832635796522238791688340428435774993627369305621598997173036310742110937147731910388435125044339176587491287063088527799489319716882716731514821040640301375746294699761566388646752213").map_err(|_|()).unwrap();
    println!("{:?}", cst191);
    let cst192 = MNT6753Fr::from_str("30619370695616348897340652990786338103356952355411400592937097493530498496575150724370795532011142747807079066635745505802585435788305542279287295289442973570497145034628148316364924730080421422583611170180839874874672292402923").map_err(|_|()).unwrap();
    println!("{:?}", cst192);
    let cst193 = MNT6753Fr::from_str("24673841218635443512615742051236228229707268092423394521962467188477230584287797759678472448727752153939143535259404892412815579547448835132857245634495118193701122601275994677173706036756819095884758213505213975282920468461648").map_err(|_|()).unwrap();
    println!("{:?}", cst193);
    let cst194 = MNT6753Fr::from_str("36572272652164880121291820457246948956941074599736210818780024032849293901174884362174417831930967055924851512065649452191662391176111111061080783830084936422625198143191870636566456310094512112645186963562699754483118474270691").map_err(|_|()).unwrap();
    println!("{:?}", cst194);
}


#[test]
fn print_cst_mds_mnt4_ord() {
    let cst_mds_mnt4_0 = MNT4753Fr::from_str("31912405202209273539598318986524890872907441719660204078020721519205822937851386704878848471397696762731294453669519820885194058788814834589747467656846993066633711579854041264897670823933422929732688429023497856398698555829506").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_0);
    let cst_mds_mnt4_1 = MNT4753Fr::from_str("36782224985949551253345987584175261651934203106046336635642725147360921512705311550363034356599663535294862100582512919892603332879823079358960531549138817036984980352080922878120484944007682806851544693860482398268494704366910").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_1);
    let cst_mds_mnt4_2 = MNT4753Fr::from_str("20189398637515961991935301345850276824345259233726855902227198300175701813517570659248975492948082493413127194435983989153803104074441799960121086049611692286455962340684607490639772939315549787357232010142288121341846054945090").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_2);
    let cst_mds_mnt4_3 = MNT4753Fr::from_str("36782224985949551253345987584175261651934203106046336635642725147360921512705311550363034356599663535294862100582512919892603332879823079358960531549138817036984980352080922878120484944007682806851544693860482398268494704366910").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_3);
    let cst_mds_mnt4_4 = MNT4753Fr::from_str("31912405202209273539598318986524890872907441719660204078020721519205822937851386704878848471397696762731294453669519820885194058788814834589747467656846993066633711579854041264897670823933422929732688429023497856398698555829506").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_4);
    let cst_mds_mnt4_5 = MNT4753Fr::from_str("34014572290828942875338754573281680115606872768810480582532234650297369811060849172129122941860272855350039930188703316395500375807915537679811040519388196242646294101212198211757319826759251546131359331178629018421055691505509").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_5);
    let cst_mds_mnt4_6 = MNT4753Fr::from_str("20189398637515961991935301345850276824345259233726855902227198300175701813517570659248975492948082493413127194435983989153803104074441799960121086049611692286455962340684607490639772939315549787357232010142288121341846054945090").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_6);
    let cst_mds_mnt4_7 = MNT4753Fr::from_str("34014572290828942875338754573281680115606872768810480582532234650297369811060849172129122941860272855350039930188703316395500375807915537679811040519388196242646294101212198211757319826759251546131359331178629018421055691505509").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_7);
    let cst_mds_mnt4_8 = MNT4753Fr::from_str("31912405202209273539598318986524890872907441719660204078020721519205822937851386704878848471397696762731294453669519820885194058788814834589747467656846993066633711579854041264897670823933422929732688429023497856398698555829506").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_8);
}

#[test]
fn print_cst_mds_mnt4_for_mont() {
    let cst_mds_mnt4_0 = MNT4753Fr::from_str("15593384137415911782").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_0);
    let cst_mds_mnt4_1 = MNT4753Fr::from_str("4797247063858472759").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_1);
    let cst_mds_mnt4_2 = MNT4753Fr::from_str("15508188267236558827").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_2);
    let cst_mds_mnt4_3 = MNT4753Fr::from_str("4797247063858472759").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_3);
    let cst_mds_mnt4_4 = MNT4753Fr::from_str("15593384137415911782").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_4);
    let cst_mds_mnt4_5 = MNT4753Fr::from_str("6372884188919678000").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_5);
    let cst_mds_mnt4_6 = MNT4753Fr::from_str("15508188267236558827").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_6);
    let cst_mds_mnt4_7 = MNT4753Fr::from_str("6372884188919678000").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_7);
    let cst_mds_mnt4_8 = MNT4753Fr::from_str("15593384137415911782").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_8);
}

#[test]
fn print_cst_mds_mnt6_for_mont() {
    let cst_mds_mnt6_0 = MNT6753Fr::from_str("18217103266694245702").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_0);
    let cst_mds_mnt6_1 = MNT6753Fr::from_str("16831488261730481765").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_1);
    let cst_mds_mnt6_2 = MNT6753Fr::from_str("6071897497319759456").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_2);
    let cst_mds_mnt6_3 = MNT6753Fr::from_str("16831488261730481765").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_3);
    let cst_mds_mnt6_4 = MNT6753Fr::from_str("18217103266694245702").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_4);
    let cst_mds_mnt6_5 = MNT6753Fr::from_str("8766094547408275164").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_5);
    let cst_mds_mnt6_6 = MNT6753Fr::from_str("6071897497319759456").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_6);
    let cst_mds_mnt6_7 = MNT6753Fr::from_str("8766094547408275164").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_7);
    let cst_mds_mnt6_8 = MNT6753Fr::from_str("18217103266694245702").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_8);
}


#[test]
//Code for printing the original representation that will be converted to Montgomery representation in zexe.
//They will end up being truncated values in zexe representation
fn print_cst_mds_mnt4_mont() {
    let cst_mds_mnt4_0 = MNT4753Fr::from_str("31912405202209273539598318986524890872907441719660204078020721519205822937851386704878848471397696762731294453669519820885194058788814834589747467656846993066633711579854041264897670823933422929732688429023497856398698555829506").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_0);
    let cst_mds_mnt4_1 = MNT4753Fr::from_str("36782224985949551253345987584175261651934203106046336635642725147360921512705311550363034356599663535294862100582512919892603332879823079358960531549138817036984980352080922878120484944007682806851544693860482398268494704366910").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_1);
    let cst_mds_mnt4_2 = MNT4753Fr::from_str("20189398637515961991935301345850276824345259233726855902227198300175701813517570659248975492948082493413127194435983989153803104074441799960121086049611692286455962340684607490639772939315549787357232010142288121341846054945090").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_2);
    let cst_mds_mnt4_3 = MNT4753Fr::from_str("36782224985949551253345987584175261651934203106046336635642725147360921512705311550363034356599663535294862100582512919892603332879823079358960531549138817036984980352080922878120484944007682806851544693860482398268494704366910").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_3);
    let cst_mds_mnt4_4 = MNT4753Fr::from_str("31912405202209273539598318986524890872907441719660204078020721519205822937851386704878848471397696762731294453669519820885194058788814834589747467656846993066633711579854041264897670823933422929732688429023497856398698555829506").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_4);
    let cst_mds_mnt4_5 = MNT4753Fr::from_str("34014572290828942875338754573281680115606872768810480582532234650297369811060849172129122941860272855350039930188703316395500375807915537679811040519388196242646294101212198211757319826759251546131359331178629018421055691505509").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_5);
    let cst_mds_mnt4_6 = MNT4753Fr::from_str("20189398637515961991935301345850276824345259233726855902227198300175701813517570659248975492948082493413127194435983989153803104074441799960121086049611692286455962340684607490639772939315549787357232010142288121341846054945090").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_6);
    let cst_mds_mnt4_7 = MNT4753Fr::from_str("34014572290828942875338754573281680115606872768810480582532234650297369811060849172129122941860272855350039930188703316395500375807915537679811040519388196242646294101212198211757319826759251546131359331178629018421055691505509").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_7);
    let cst_mds_mnt4_8 = MNT4753Fr::from_str("31912405202209273539598318986524890872907441719660204078020721519205822937851386704878848471397696762731294453669519820885194058788814834589747467656846993066633711579854041264897670823933422929732688429023497856398698555829506").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_8);
}


#[test]
//Code for printing the original representation that will be converted to Montgomery representation in zexe.
//They will end up being truncated values in zexe representation
fn print_cst_mds_mnt6_for_partial_mont() {
    let cst_mds_mnt6_0 = MNT6753Fr::from_str("25172171321756700709860226051956409175097766788035088105114088511827035832832957786755581017417036395211876759845677129230917517652276192015588529510543710125311029914012194389596176556880994788982747792692944817124281853157681").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_0);
    let cst_mds_mnt6_1 = MNT6753Fr::from_str("36033183667475609016229993434557782174826671238820686024637113708698362654488024890960814097939689468367231150786375645263532295732482914294227552594526047879826055486462765598029061538565382258731489144801341806834148591238457").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_1);
    let cst_mds_mnt6_2 = MNT6753Fr::from_str("17584523493017126959883022470593411310160931579228883880094116706069059240688813057775932271156496740610568791195984900263099105686103794679828249206130595498109810142793768370919405048756530577033171075816910073382149330619704").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_2);
    let cst_mds_mnt6_3 = MNT6753Fr::from_str("36033183667475609016229993434557782174826671238820686024637113708698362654488024890960814097939689468367231150786375645263532295732482914294227552594526047879826055486462765598029061538565382258731489144801341806834148591238457").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_3);
    let cst_mds_mnt6_4 = MNT6753Fr::from_str("25172171321756700709860226051956409175097766788035088105114088511827035832832957786755581017417036395211876759845677129230917517652276192015588529510543710125311029914012194389596176556880994788982747792692944817124281853157681").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_4);
    let cst_mds_mnt6_5 = MNT6753Fr::from_str("40853304099076078130051168302276915452719994190347889584022389193738864356755624397719620313974170388204907879992203725535175837165514937822725297339754522617652585343869661985136395643645826734086113807574380809237019014592608").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_5);
    let cst_mds_mnt6_6 = MNT6753Fr::from_str("17584523493017126959883022470593411310160931579228883880094116706069059240688813057775932271156496740610568791195984900263099105686103794679828249206130595498109810142793768370919405048756530577033171075816910073382149330619704").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_6);
    let cst_mds_mnt6_7 = MNT6753Fr::from_str("40853304099076078130051168302276915452719994190347889584022389193738864356755624397719620313974170388204907879992203725535175837165514937822725297339754522617652585343869661985136395643645826734086113807574380809237019014592608").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_7);
    let cst_mds_mnt6_8 = MNT6753Fr::from_str("25172171321756700709860226051956409175097766788035088105114088511827035832832957786755581017417036395211876759845677129230917517652276192015588529510543710125311029914012194389596176556880994788982747792692944817124281853157681").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_8);
}

#[test]
fn print_cst_mds_mnt4_mont_original() {
    let cst_mds_mnt4_0 = MNT4753Fr::from_str("15593384137415911782").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_0);
    let cst_mds_mnt4_1 = MNT4753Fr::from_str("4797247063858472759").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_1);
    let cst_mds_mnt4_2 = MNT4753Fr::from_str("15508188267236558827").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_2);
    let cst_mds_mnt4_3 = MNT4753Fr::from_str("4797247063858472759").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_3);
    let cst_mds_mnt4_4 = MNT4753Fr::from_str("15593384137415911782").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_4);
    let cst_mds_mnt4_5 = MNT4753Fr::from_str("6372884188919678000").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_5);
    let cst_mds_mnt4_6 = MNT4753Fr::from_str("15508188267236558827").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_6);
    let cst_mds_mnt4_7 = MNT4753Fr::from_str("6372884188919678000").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_7);
    let cst_mds_mnt4_8 = MNT4753Fr::from_str("15593384137415911782").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_8);
}


#[test]
// Code for printing the original representation that will be converted to Montgomery representation in zexe.
// These constants are for use with the conventional Montgomery representation
fn print_cst_mds_mnt4_mont_adjusted() {
    let cst_mds_mnt4_0 = MNT4753Fr::from_str("32162258476357702564129695379244120200683826515235163854151969033894744506248543898308258132181264283290923651253507942218918122782381002955316774929927064705185500064658330620364481175095600705209681752787775465803230999312886").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_0);
    let cst_mds_mnt4_1 = MNT4753Fr::from_str("26608317340275702924884808491736298870519837311191442624508248689006743200175859396788281823617186909978078747338485030768087786187171523386857619937694283171712729669957997143289593200423337490665196023345908494426340361079767").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_1);
    let cst_mds_mnt4_2 = MNT4753Fr::from_str("13427748741431160535336715211524173688804768448922025825917412478156339401883060150848161898056035566871662282814923364767902323728805437491988718293028041789304651378852479723741982425565515669181406705439696666589033512318189").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_2);
    let cst_mds_mnt4_3 = MNT4753Fr::from_str("26608317340275702924884808491736298870519837311191442624508248689006743200175859396788281823617186909978078747338485030768087786187171523386857619937694283171712729669957997143289593200423337490665196023345908494426340361079767").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_3);
    let cst_mds_mnt4_4 = MNT4753Fr::from_str("32162258476357702564129695379244120200683826515235163854151969033894744506248543898308258132181264283290923651253507942218918122782381002955316774929927064705185500064658330620364481175095600705209681752787775465803230999312886").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_4);
    let cst_mds_mnt4_5 = MNT4753Fr::from_str("21777152745044010423909873657073026887168462082432061564557048777395318020860183576317633332610326094913375510601192754757458279972726174420292636369212955306298594925284840993120169665637403416508160253557899354429149607289195").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_5);
    let cst_mds_mnt4_6 = MNT4753Fr::from_str("13427748741431160535336715211524173688804768448922025825917412478156339401883060150848161898056035566871662282814923364767902323728805437491988718293028041789304651378852479723741982425565515669181406705439696666589033512318189").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_6);
    let cst_mds_mnt4_7 = MNT4753Fr::from_str("21777152745044010423909873657073026887168462082432061564557048777395318020860183576317633332610326094913375510601192754757458279972726174420292636369212955306298594925284840993120169665637403416508160253557899354429149607289195").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_7);
    let cst_mds_mnt4_8 = MNT4753Fr::from_str("32162258476357702564129695379244120200683826515235163854151969033894744506248543898308258132181264283290923651253507942218918122782381002955316774929927064705185500064658330620364481175095600705209681752787775465803230999312886").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt4_8);
}

#[test]
// Code for printing the original representation that will be converted to Montgomery representation in zexe.
// These constants are for use with the conventional Montgomery representation
fn print_cst_mds_mnt6_mont_adjusted() {
    let cst_mds_mnt6_0 = MNT6753Fr::from_str("14224748791865839125967673899911916281423111984497001831433070383692701340596394126700408302119265665928503566389826607706751102302441146867437412443905619797606809320199336864874745888994173859923322571774565004724919588266390").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_0);
    let cst_mds_mnt6_1 = MNT6753Fr::from_str("41618057895389570755116538835629147681590819837313024147725801231306921869933189666098378001564307145623712461246804836497883124793740730294192196096820644436519309514816212079857431184976574608592240335816090471232135946564440").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_1);
    let cst_mds_mnt6_2 = MNT6753Fr::from_str("5205911091534147964783147468420370053643133629684203566962382030710704597874067604175047092647943631191574793749228244410668768332193317313989133431866247482010150727354090179236248499319912822011571938212472907839008542777541").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_2);
    let cst_mds_mnt6_3 = MNT6753Fr::from_str("41618057895389570755116538835629147681590819837313024147725801231306921869933189666098378001564307145623712461246804836497883124793740730294192196096820644436519309514816212079857431184976574608592240335816090471232135946564440").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_3);
    let cst_mds_mnt6_4 = MNT6753Fr::from_str("14224748791865839125967673899911916281423111984497001831433070383692701340596394126700408302119265665928503566389826607706751102302441146867437412443905619797606809320199336864874745888994173859923322571774565004724919588266390").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_4);
    let cst_mds_mnt6_5 = MNT6753Fr::from_str("4118402267432274907784787300396781986780228566299527692712362225596064104795244945118048765619857159034480412498106451775503405200887387573595968455178716049471618008849989036370971213453272016629692939662613543081574586930569").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_5);
    let cst_mds_mnt6_6 = MNT6753Fr::from_str("5205911091534147964783147468420370053643133629684203566962382030710704597874067604175047092647943631191574793749228244410668768332193317313989133431866247482010150727354090179236248499319912822011571938212472907839008542777541").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_6);
    let cst_mds_mnt6_7 = MNT6753Fr::from_str("4118402267432274907784787300396781986780228566299527692712362225596064104795244945118048765619857159034480412498106451775503405200887387573595968455178716049471618008849989036370971213453272016629692939662613543081574586930569").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_7);
    let cst_mds_mnt6_8 = MNT6753Fr::from_str("14224748791865839125967673899911916281423111984497001831433070383692701340596394126700408302119265665928503566389826607706751102302441146867437412443905619797606809320199336864874745888994173859923322571774565004724919588266390").map_err( |_|()).unwrap();
    println!("{:?}", cst_mds_mnt6_8);
}