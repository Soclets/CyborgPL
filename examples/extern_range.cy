# A ranged extern return is a contract checked at the door (decided 2026-09-22): outside it, the call fails out_of_range.
%extern.C proc strlen [$s: ptr[u8]] -> u64[min: 0, max: 10];      # C promises at most 10; checked at the door

MAIN:
    let $short := "hello";
    let $long := "this is longer than ten";
    no_check let $a := strlen(lend $short) ?? 99;
    no_check let $b := strlen(lend $long) ?? 99;    # 23 is outside 0..10: fails out_of_range
    let $c: i8 := $a;                                    # proven by the bound: at most 10
    std::print($a, " ", $b, " ", $c, newln);
    no_check let $d := strlen(lend $long)-
    otherwise:
        case out_of_range:
            std::print("C lied", newln); std::exit(code: 3);
        ;case out_of_range
    ;otherwise;
;MAIN
