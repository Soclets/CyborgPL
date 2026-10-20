# `std::print` and `std::exit` with no OS (owner, 2026-10-09: "f", "`a`",
# "c"): the bytes go to the board Neuron's serial port, and `exit` powers
# the machine off. Printed as on an OS, with no heap.
struct Point {
    x: i32,
    y: i32
}

MAIN {
    let 'p' := Point[x = 3, y = -4];
    let 'tenth': f64 = 0.1;
    std::print.useful["hello from a bare chip", newln];
    std::print.useful['p', " ", 'tenth', " ", 7 times 6, newln];
    std::print.stored['tenth', newln];
    # A file's bytes, read in place from the program: no heap (owner, 2026-10-10).
    const 'tiny' := incl.bytes["assets/tiny.bin"];
    std::print.useful[array.length['tiny'], " ", 'tiny'[pos 4], newln];
    std::exit[0];
}
