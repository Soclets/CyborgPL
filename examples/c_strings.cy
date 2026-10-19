# C strings and a C function under another name (owner, 2026-10-19). `C.str.make` makes the bytes and a 0 C reads,
# failing `has_zero_byte` (a literal cannot); `C.str.read` reads one back, failing `not_utf8`. `= C.printf` gives C's
# printf a name of ours: the open form takes any extras, a fixed shape exactly the ones it names. Every C call is
# `no_check` (a fixed shape's extras are still type-checked). C's format text is bytes: no `\n` escape here.
extern.C fn c_printf = C.printf ['format': ptr[uint8], ...'args'] -> int32;
extern.C fn printf_count = C.printf ['format': ptr[uint8], ...'n': int32] -> int32;
extern.C fn strlen ['s': ptr[uint8]] -> uint64;

MAIN {
    let 'name' := "Tankun";
    let 'cs' := C.str.make[lend 'name'] otherwise { has_zero_byte { std::exit[1]; } };
    let 'fmt' := C.str.make["%d apples for %s, "];
    no_check c_printf[lend 'fmt', 5, lend 'cs'];
    let 'one' := C.str.make["%d pears, "];
    no_check printf_count[lend 'one', 7];
    no_check let 'n' := strlen[lend 'cs'];
    let 'back' := C.str.read[lend 'cs'] otherwise "?";
    std::print.useful['n', " ", 'back', newln];
}
