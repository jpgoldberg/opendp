from opendp.trans import *
from opendp.mod import enable_features
enable_features("contrib")

def test_make_bounded_float_sequential_sum():
    sum_trans = make_bounded_float_sequential_sum(100, (0., 10.))
    print(sum_trans([1., 2., 4.]))

def test_make_sized_bounded_float_sequential_sum():
    sum_trans = make_sized_bounded_float_sequential_sum(3, (0., 10.))
    print(sum_trans([1., 2., 4.]))

