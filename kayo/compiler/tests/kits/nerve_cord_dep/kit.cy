# Lists the built-in cord, and is built beside a Neuron listing the Nerve one: one cord per program
# (owner, 2026-10-14), so its `std::twice` is the Nerve cord's.

pub fn doubled [let $x: i64] -> i64:
    ret std::twice($x);
;fn doubled
