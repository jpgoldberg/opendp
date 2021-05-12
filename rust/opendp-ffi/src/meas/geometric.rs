use std::os::raw::{c_char, c_void};

use num::{CheckedAdd, CheckedSub, Float, Zero};

use opendp::err;
use opendp::samplers::SampleGeometric;
use opendp::traits::DistanceCast;
use opendp::meas;

use crate::core::{FfiMeasurement, FfiResult};
use crate::util::{parse_type_args, into_raw};

#[no_mangle]
pub extern "C" fn make_base_simple_geometric(
    type_args: *const c_char, scale: *const c_void, min: *const c_void, max: *const c_void,
) -> *mut FfiResult<*mut FfiMeasurement> {
    fn monomorphize<T, QO>(scale: *const c_void, min: *const c_void, max: *const c_void) -> FfiResult<*mut FfiMeasurement>
        where T: 'static + Clone + SampleGeometric + CheckedSub<Output=T> + CheckedAdd<Output=T> + DistanceCast + Zero,
              QO: 'static + Float + DistanceCast, f64: From<QO> {
        let scale = *try_as_ref!(scale as *const QO);
        let min = try_as_ref!(min as *const T).clone();
        let max = try_as_ref!(max as *const T).clone();
        meas::make_base_geometric::<T, QO>(scale, min, max).into()
    }
    let type_args = try_raw!(parse_type_args(type_args, 2));
    into_raw(dispatch!(monomorphize, [(type_args[0], @integers), (type_args[1], @floats)], (scale, min, max)))
}