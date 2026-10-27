# C strings and a C function under another name (owner, 2026-10-19). `C.str.make` makes the bytes and a 0 C reads,
# failing `has_zero_byte` (a literal cannot); `C.str.read` reads one back, failing `not_utf8`. `= C.printf` gives C's
# printf a name of ours: the open form takes any extras, a fixed shape exactly the ones it names. Every C call is
# `no_check` (a fixed shape's extras are still type-checked). C's format text is bytes: no `\n` escape here.
%external.C procedure c_printf = %C.printf [$format: pointer[integer.unsigned.8_bit], ...$args] -> integer.signed.32_bit;
%external.C procedure printf_count = %C.printf [$format: pointer[integer.unsigned.8_bit], ...$n: integer.signed.32_bit] -> integer.signed.32_bit;
%external.C procedure strlen [$s: pointer[integer.unsigned.8_bit]] -> integer.unsigned.64_bit;

!MAIN:
    let $name := "Tankun";
    let $cs := %C.string.make(lend $name) otherwise:
        case has_zero_byte:
            std::exit(code: 1);
        end of case has_zero_byte
    end of otherwise;
    let $fmt := %C.string.make("%d apples for %s, ");
    !no_check c_printf(lend $fmt, 5, lend $cs);
    let $one := %C.string.make("%d pears, ");
    !no_check printf_count(lend $one, 7);
    !no_check let $n := strlen(lend $cs);
    let $back := %C.string.read(lend $cs) otherwise "?";
    std::print.shortest($n, " ", $back, newline);
    # a C string's type is `C.str`; a bounded one says whether its 0 counts (owner, 2026-10-19)
    let $fits: %C.string[maximum: 6 bytes, exclude zero] = %C.string.make(lend $name) otherwise:
        case has_zero_byte:
            std::exit(code: 1);
        end of case has_zero_byte
        case out_of_range:
            std::exit(code: 2);
        end of case out_of_range
    end of otherwise;
    let $buffer: %C.string[maximum: 6 bytes, include zero] = %C.string.make(lend $name) otherwise:
        case has_zero_byte:
            std::exit(code: 1);
        end of case has_zero_byte
        case out_of_range:
            std::print.shortest("Tankun and its 0 need 7 bytes", newline); std::exit(code: 0);
        end of case out_of_range
    end of otherwise;
!end of MAIN
