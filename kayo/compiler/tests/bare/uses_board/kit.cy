# `std::print` and `std::exit` with no OS (owner, 2026-10-09: "f", "`a`",
# "c"): the bytes go to the board Neuron's serial port, and `exit` powers
# the machine off. Printed as on an OS, with no heap.
rec Point:
    var x: i32;
    var y: i32;
;rec Point

MAIN:
    let $p := Point[x = 3, y = -4];
    let $tenth: f64 := 0.1;
    std::print("hello from a bare chip", newln);
    std::print($p, " ", $tenth, " ", 7 x 6, newln);
    std::print.stored($tenth, newln);
    # A file's bytes, read in place from the program: no heap (owner, 2026-10-10).
    const $tiny := incl.bytes["assets/tiny.bin"];
    std::print(arr.len($tiny), " ", $tiny[pos: 4], newln);
    std::exit(code: 0);
;MAIN
