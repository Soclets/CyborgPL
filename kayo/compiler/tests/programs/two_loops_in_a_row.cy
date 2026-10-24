# Layer 3 walked its unrolled loops in the order it found them. With two
# loops in a row, the second loop's last trip was written down before the
# first loop's way out had arrived there, so the site in it was asked only
# on a path that cannot happen, and "proven" came back (check claude,
# seed 133). `0 - 'n'` with 'n' = 2 is -2, which no `u64` holds: storing
# it must stop. (Since plain math is exact, the site is the store.)
extern.C fn strlen [$s: ptr[u8]] -> u64;
MAIN {
    let $s := "ab";
    no_check let $n: u64 = strlen(lend $s);
    for $i: i16 [from: 37, to: 40, by: 1] {
    }
    let mut $w: u16 = 4;
    loop {
        if $w at_most 0 { break; }
        change $w = $w minus 1;
        let $v: u64 = 0 minus $n;
    }
}
