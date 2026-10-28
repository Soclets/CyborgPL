# A brain16's useful decimal is its shortest form even when large (check claude, seed 25294): 2^63
# as a brain16 is 9220000000000000000, which fits an i64, not its stored 9223372036854775808.
MAIN:
    let $i: i64 := 9223372036854775803;
    let $b := to.%bf16($i);
    let $d := to.shortest.%d64($b);
    let $n := to.i64($d) ?? -1;
    std::print($b, " ", $d, " ", $n, newln);
;MAIN
