# Lists the built-in cord, and is built beside a Neuron listing the Nerve one: one cord per program
# (owner, 2026-10-14), so its `std::twice` is the Nerve cord's.

pub bare.never fn doubled ['x': i64] -> i64 {
    return std::twice['x'];
}
