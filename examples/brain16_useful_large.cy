# A brain16's useful decimal is its shortest form even when large (check claude, seed 25294): 2^63
# as a brain16 is 9220000000000000000, which fits an i64, not its stored 9223372036854775808.
!MAIN:
    let $i: integer.signed.64_bit = 9223372036854775803;
    let $b := to.float.%brain.16_bit($i);
    let $d := to.shortest.float.decimal.64_bit($b);
    let $n := to.integer.signed.64_bit($d) otherwise negative 1;
    std::print.shortest($b, " ", $d, " ", $n, newline);
!end of MAIN
