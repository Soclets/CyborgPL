# `std::exit` with a code worked out, not a name: exact arithmetic holds it wider than the `u8` it is, and the
# exit took that width as it was (LLVM: "Type too small for ZExt"). 3 + 1, then + 4: exit 8.
MAIN:
    var $a: i32 = 3;
    change $a to $a + 1;
    std::exit(code: $a + 4);
;MAIN
