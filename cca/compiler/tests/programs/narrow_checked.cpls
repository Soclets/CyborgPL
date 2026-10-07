# Not in expected.txt: what it exits with depends on who runs it. The test
# reads its IR — an unproven narrowing has to reach the binary as a check.
extern.C fn exit ['code': int32];
extern.C fn getuid [] -> uint32;
MAIN {
    no_check let 'u' := getuid[];
    let 'b': uint8 = 'u';
    no_check my.exit[to.int32['b']];
}
