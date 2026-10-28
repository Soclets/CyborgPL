# C strings and a C function under another name (owner, 2026-10-19). `C.str.make` makes the bytes and a 0 C reads,
# failing `has_zero_byte` (a literal cannot); `C.str.read` reads one back, failing `not_utf8`. `= C.printf` gives C's
# printf a name of ours: the open form takes any extras, a fixed shape exactly the ones it names. Every C call is
# `no_check` (a fixed shape's extras are still type-checked). C's format text is bytes: no `\n` escape here.
%extern.C proc c_printf = %C.printf [let $format: ptr[u8], let ...$args] -> i32;
%extern.C proc printf_count = %C.printf [let $format: ptr[u8], let ...$n: i32] -> i32;
%extern.C proc strlen [let $s: ptr[u8]] -> u64;

MAIN:
    let $name := "Tankun";
    let $cs := %C.str.make(lend $name)-
    otherwise:
        case has_zero_byte:
            std::exit(code: 1);
        ;case has_zero_byte
    ;otherwise;
    let $fmt := %C.str.make("%d apples for %s, ");
    no_check c_printf(lend $fmt, 5, lend $cs);
    let $one := %C.str.make("%d pears, ");
    no_check printf_count(lend $one, 7);
    no_check let $n := strlen(lend $cs);
    let $back := %C.str.read(lend $cs) ?? "?";
    std::println($n, " ", $back);
    # a C string's type is `C.str`; a bounded one says whether its 0 counts (owner, 2026-10-19)
    let $fits: %C.str[max: 6 bytes, exclude zero] := %C.str.make(lend $name)-
    otherwise:
        case has_zero_byte:
            std::exit(code: 1);
        ;case has_zero_byte
        case out_of_range:
            std::exit(code: 2);
        ;case out_of_range
    ;otherwise;
    let $buffer: %C.str[max: 6 bytes, include zero] := %C.str.make(lend $name)-
    otherwise:
        case has_zero_byte:
            std::exit(code: 1);
        ;case has_zero_byte
        case out_of_range:
            std::println("Tankun and its 0 need 7 bytes"); std::exit(code: 0);
        ;case out_of_range
    ;otherwise;
;MAIN
