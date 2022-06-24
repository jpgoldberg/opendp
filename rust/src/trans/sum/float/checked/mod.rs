use std::iter::Sum;

use num::One;

use crate::{
    core::{Function, StabilityRelation, Transformation},
    dist::{AbsoluteDistance, IntDistance, SymmetricDistance},
    dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain},
    error::Fallible,
    traits::{
        AlertingAbs, CheckNull, DistanceConstant, ExactIntCast, FloatBits, InfAdd, InfCast, InfPow,
        InfSub,
    }, samplers::Shuffle,
};

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_sized_bounded_float_checked_sum<T>(
    size: usize,
    bounds: (T, T),
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<BoundedDomain<T>>>,
        AllDomain<T>,
        SymmetricDistance,
        AbsoluteDistance<T>,
    >,
>
where
    T: DistanceConstant<IntDistance>
        + ExactIntCast<T::Bits>
        + InfAdd
        + InfPow
        + One
        + FloatBits
        + ExactIntCast<usize>
        + InfSub
        + AlertingAbs
        + CheckNull,
    for<'a> T: Sum<&'a T>,
    Vec<T>: Shuffle,
    IntDistance: InfCast<T>,
{
    let size_ = T::exact_int_cast(size)?;
    let (lower, upper) = bounds.clone();

    lower
        .inf_mul(&size_)
        .or(upper.inf_mul(&size_))
        .map_err(|_| {
            err!(
                MakeTransformation,
                "potential for overflow when computing function"
            )
        })?;

    let mantissa_bits = T::exact_int_cast(T::MANTISSA_BITS)?;
    let _2 = T::one().inf_add(&T::one())?;

    let relaxation = size_
        .inf_mul(&size_)?
        .inf_div(&_2.inf_pow(&mantissa_bits)?)?;
    let ideal_sensitivity = upper.inf_sub(&lower)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new_fallible(move |arg: &Vec<T>| {
            let mut data = arg.clone();
            if arg.len() > size { data.shuffle()? }
            Ok(arg.iter().take(size).sum())
        }),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_forward(move |d_in: &IntDistance| {
            T::inf_cast(*d_in / 2)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&relaxation)
        }),
    ))
}