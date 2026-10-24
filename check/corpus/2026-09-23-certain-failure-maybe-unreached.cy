# env CHK_A 3
# Owner's ruling 2026-09-23 (design/precog.md layer 2): a site no value can
# pass, in a branch that may not run, is unproven, not a compile error.
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
extern.C fn strlen ['s': ptr[u8]] -> u64;
MAIN {
    let 'k' := "CHK_A";
    no_check let 'n': u64 = strlen(getenv(lend 'k'));
    let 'p': i16 = -5;
    if 'n' > 1000000 {
        let 'x': u64 = to.u64('p');
    }
    let 'hi' := "ran on";
    no_check write(1, lend 'hi', 6);
}
