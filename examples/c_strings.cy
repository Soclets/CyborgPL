# C strings and a C function under another name (owner, 2026-10-19). `C.str.make` makes the bytes and a 0 C reads,
# failing `has_zero_byte` (a literal cannot); `C.str.read` reads one back, failing `not_utf8`. `= C.printf` gives C's
# printf a name of ours: the open form takes any extras, a fixed shape exactly the ones it names. Every C call is
# `no_check` (a fixed shape's extras are still type-checked). C's format text is bytes: no `\n` escape here.
extern.C fn c_printf = C.printf ['format': ptr[u8], ...'args'] -> i32;
extern.C fn printf_count = C.printf ['format': ptr[u8], ...'n': i32] -> i32;
extern.C fn strlen ['s': ptr[u8]] -> u64;

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
    # a C string's type is `C.str`; a bounded one says whether its 0 counts (owner, 2026-10-19)
    let 'fits': C.str[max 6 bytes, exclude zero] = C.str.make[lend 'name'] otherwise { has_zero_byte { std::exit[1]; } out_of_range { std::exit[2]; } };
    let 'buffer': C.str[max 6 bytes, include zero] = C.str.make[lend 'name'] otherwise { has_zero_byte { std::exit[1]; } out_of_range { std::print.useful["Tankun and its 0 need 7 bytes", newln]; std::exit[0]; } };
}
