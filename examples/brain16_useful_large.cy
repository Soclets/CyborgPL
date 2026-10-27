# A brain16's useful decimal is its shortest form even when large (check claude, seed 25294): 2^63
# as a brain16 is 9220000000000000000, which fits an i64, not its stored 9223372036854775808.
!MAIN:
    let $i: int.sign.64 = 9223372036854775803;
    let $b := to.float.%brain.16($i);
    let $d := to.shortest.float.deci.64($b);
    let $n := to.int.sign.64($d) otherwise neg 1;
    std::print.shortest($b, " ", $d, " ", $n, newln);
!end of MAIN
