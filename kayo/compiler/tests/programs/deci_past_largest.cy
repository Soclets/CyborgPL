# A deci answer past the largest value stops the program where it is
# (owner, 2026-09-29; the oracle: "does not fit d32: past the largest
# value"): 9999999 to the 15th is past 10^96. The exit before it is never
# reached.
extern.C fn exit ['code': i32];
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
MAIN {
    let 'm': d32 = 9999999;
    let mut 'q': d32 = 'm';
    for 'i': i32 from 1 to 15 by 1 {
        change 'q' = 'q' times 'm';
    }
    no_check my.exit[7];
}
