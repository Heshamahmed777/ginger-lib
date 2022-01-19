use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use std::ops::{AddAssign, MulAssign, SubAssign};

use algebra::{
    biginteger::{BigInteger256 as FrRepr, BigInteger256 as FqRepr},
    fields::tweedle::{fq::Fq, fr::Fr},
    BigInteger, Field, PrimeField, ProjectiveCurve, SquareRootField, UniformRand,
};

ec_bench!();
f_bench!(Fq, Fq, FqRepr, FqRepr, fq);
f_bench!(Fr, Fr, FrRepr, FrRepr, fr);
