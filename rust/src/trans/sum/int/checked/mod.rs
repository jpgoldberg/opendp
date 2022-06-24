use std::iter::Sum;

use crate::{
    error::Fallible, 
    core::{Transformation, Function, StabilityRelation}, 
    dom::{VectorDomain, BoundedDomain, AllDomain, SizedDomain}, 
    dist::{SymmetricDistance, AbsoluteDistance, IntDistance}, 
    traits::{DistanceConstant, CheckNull, InfCast, ExactIntCast, InfSub, InfDiv}
};

use super::AddIsExact;

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_sized_bounded_int_checked_sum<T>(
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
        + ExactIntCast<usize>
        + InfSub
        + CheckNull
        + InfDiv
        + AddIsExact,
    for<'a> T: Sum<&'a T>,
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

    let range = upper.inf_sub(&lower)?;
    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().sum()),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_forward(
            // If d_in is odd, we still only consider databases with (d_in - 1) / 2 substitutions,
            //    so floor division is acceptable
            move |d_in: &IntDistance| T::inf_cast(d_in / 2).and_then(|d_in| d_in.inf_mul(&range)),
        ),
    ))
}
