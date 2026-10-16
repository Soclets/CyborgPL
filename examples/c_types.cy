# C's machine-sized types and enums (owner, 2026-10-10). `C.int`, `C.long`, `C.char`, `C.size_t`,
# `C.long_double` … are what C's types are on the machine built for (here 32, 64, 8 signed, 64 bits, and
# a `bin64`); a header comes through in them, and a type the same size everywhere is CyborgPL's own
# (`int32_t` is `int32`). A named C enum is a `layout.C enum` (`color.BLUE`, its value 6, counted on
# from `GREEN = 5`); one named by its `typedef` takes that name (`level_t`); an unnamed one's names are
# consts (`'SEEK_B'`), and so is a later name for a number already named (`'SHADE_LAST'`). A C function pointer is CyborgPL's callback type, `fn[…] -> C.int`. All are CNC's.
incl.C["headers/paint.h"];

MAIN {
    let 'n': C.long = 40;
    let 'c': C.char = -3;
    let 'd': C.long_double = 1.5;
    std::print.useful[to.int32[color.BLUE], " ", to.int32[level_t.LOW], " ", 'SEEK_B', newln];
    std::print.useful['n' plus 2, " ", 'c', " ", 'd', " ", size.of[C.long], " ", size.of[C.int], newln];
    # a header's last struct comes through whole too (compiler claude found it opaque once): 16 bytes
    std::print.useful[size.of[last], newln];
    # an unnamed struct named by its `typedef` comes through whole as well: `pair_t`, 8 bytes
    std::print.useful[size.of[pair_t], newln];
    # two names for one number (owner, 2026-10-12: "(a)"): the first is the case, a later one a const,
    # `SHADE_LAST = LIGHT` giving `'SHADE_LAST'`, 2; a number becomes the enum through `to.`, checked
    let 's' := to.shade['SHADE_LAST'] otherwise shade.DARK;
    std::print.useful['SHADE_FIRST', " ", 'SHADE_LAST', " ", to.int32['s'], newln];
}
