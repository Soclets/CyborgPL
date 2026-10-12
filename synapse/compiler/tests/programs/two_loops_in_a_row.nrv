# Layer 3 walked its unrolled loops in the order it found them. With two
# loops in a row, the second loop's last trip was written down before the
# first loop's way out had arrived there, so the site in it was asked only
# on a path that cannot happen, and "proven" came back (check claude,
# seed 133). `0 - 'n'` with 'n' = 2 is -2, which no `uint64` holds: storing
# it must stop. (Since plain math is exact, the site is the store.)
extern.C fn strlen ['s': ptr[uint8]] -> uint64;
MAIN {
    let 's' := "ab";
    no_check let 'n': uint64 = strlen[lend 's'];
    for 'i': int16 from 37 to 40 by 1 {
    }
    let mut 'w': uint16 = 4;
    loop {
        if 'w' at_most 0 { break; }
        change 'w' = 'w' minus 1;
        let 'v': uint64 = 0 minus 'n';
    }
}
