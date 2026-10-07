# `lend.mut 'x' -> x1` hands the assembly the address of 'x' to write
# through (owner, 2026-09-30: a). 'x' was 1 and is 100 after it, so
# `'x' x 3` is 300, which no uint8 holds: the program stops. Precog must
# not keep what it knew of 'x' before the assembly.
extern.C fn exit ['code': int32];
MAIN {
    let mut 'x': int64 = 1;
    no_check inline.ASM {
        mov x2, #100
        str x2, [x1]
    } with [lend.mut 'x' -> x1, clobber x2];
    let 'y': uint8 = to.uint8['x' times 3];
    no_check my.exit[to.int32['y']];
}
