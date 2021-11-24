use std::{
    cmp::{Ord, Ordering, PartialOrd},
    fmt::{Display, Formatter, Result as FmtResult},
    marker::PhantomData,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    str::FromStr,
};
use unroll::unroll_for_loops;

use crate::{
    biginteger::{
        arithmetic as fa, BigInteger as _BigInteger, BigInteger256, 
        BigInteger320, BigInteger384,
        BigInteger768, BigInteger832,
    },
    bytes::{FromBytes, ToBytes},
    fields::{
        Field, FpParameters, LegendreSymbol, MulShort, MulShortAssign, PrimeField, SquareRootField,
    },
    serialize::{
        buffer_byte_size, CanonicalDeserialize, CanonicalDeserializeWithFlags, CanonicalSerialize,
        CanonicalSerializeWithFlags, EmptyFlags, Flags, SerializationError,
    },
    SemanticallyValid, 
};

use serde::{Deserialize, Serialize};
use std::io::{Error as IoError, ErrorKind, Read, Result as IoResult, Write};

#[cfg(use_asm)]
use std::mem::MaybeUninit;

#[cfg(use_asm)]
include!(concat!(env!("OUT_DIR"), "/field_assembly.rs"));

impl_Fp!(Fp256, Fp256Parameters, BigInteger256, BigInteger256, 4, 2);
impl_Fp!(Fp320, Fp320Parameters, BigInteger320, BigInteger320, 5, 0);
impl_Fp!(Fp384, Fp384Parameters, BigInteger384, BigInteger384, 6, 0);
impl_Fp!(Fp768, Fp768Parameters, BigInteger768, BigInteger768, 12, 0);
impl_Fp!(Fp832, Fp832Parameters, BigInteger832, BigInteger832, 13, 0);

pub mod fp2;
pub use self::fp2::*;

pub mod fp3;
pub use self::fp3::*;

pub mod fp4;
pub use self::fp4::*;

pub mod fp6_2over3;
pub use self::fp6_2over3::*;

pub mod fp6_3over2;

pub mod fp12_2over3over2;

pub mod quadratic_extension;
pub use quadratic_extension::*;

pub mod cubic_extension;
pub use cubic_extension::*;
