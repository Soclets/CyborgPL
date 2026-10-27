# `NO_FUCKING_CHECKS` (owner, 2026-09-27): the fourth tier. Everything `no_check` turns off, and
# what C hands back is trusted too, with no `out_of_range` check on a C enum or range. Goes
# where `no_check` goes, a statement or a block, never a file; listed like `no_check`.
%external.C procedure write [$fd: integer.signed.32_bit, $buf: pointer[integer.unsigned.8_bit], $len: integer.unsigned.64_bit] -> integer.signed.64_bit;
!MAIN:
    let $msg := "hello";
    !NO_FUCKING_CHECKS let $n := write(1, lend $msg, std::string.bytes(lend $msg));
    !NO_FUCKING_CHECKS:
        let $a: integer.signed.32_bit = 10;
        let $b: integer.signed.32_bit = 2;
        let $q := $a divide.floor $b;
        std::print.shortest(newline, $q, newline);
    !end of NO_FUCKING_CHECKS
!end of MAIN
