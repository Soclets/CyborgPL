# A write through `for 'x' in lend.mut 'xs'` changes the list, for Precog
# too. It wrote only 'x', kept the list's 2s, and "proved" that
# `'xs'[1] x 50` fits an i8 — the list holds 6s, and 300 does not: the
# program stops, as the oracle does.
%extern.C proc exit [$code: i32];
MAIN:
    var $xs: std::ls[i64] = std::ls.fill(value: 2, count: 3);
    for $x in lend.mut $xs:
        change $x to $x x 3;
    ;for
    let $y: i8 := to.i8($xs[pos: 1] x 50);
    no_check my.exit(to.i32($y));
;MAIN
