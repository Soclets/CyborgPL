# A bin32 that can be anything the type holds, stored: Precog widened its
# range a step past the largest, to infinity, and `0.0 x 'one'` read as
# "0 x -inf has no number, every time" — the build refused a program that
# always works. Past the largest is the store's own site to stop, so the
# range stays within it. 0 x 1 is 0: exit 7.
extern.C fn exit ['code': int32];
extern.C fn strlen ['s': ptr[uint8]] -> uint64;
MAIN {
    let 's' := "x";
    no_check let 'n': uint64 = strlen[lend 's'];
    let 'z': int32 = to.int32['n'] minus 1;
    let 'one': bin32 = to.bin32[1 plus 'z'];
    let 'x': bin32 = 0.0 times 'one';
    let mut 'code': int32 = 0;
    if 'x' equal_to 0.0 { change 'code' = 7; }
    no_check my.exit['code'];
}
