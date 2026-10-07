# `and.then` / `or.else` stop once the answer is known (owner, 2026-09-25), so
# the check on the left guards the read on the right: Precog proves each read
# in range, and the reads that would be out of range never run.
extern.C fn exit ['code': int32];

MAIN {
    let 'xs' := [3, 5, 8];
    let mut 'i': uint64 = 0;
    let mut 'hits': int32 = 0;
    let mut 'kept': int32 = 0;
    while 'i' < 10 {
        if 'i' at_least 1 and.then 'i' at_most list.length[lend 'xs'] and.then 'xs'[pos 'i'] equal_to 5 {
            change 'hits' = 'hits' plus 1;
        }
        if 'i' equal_to 0 or.else 'i' > list.length[lend 'xs'] or.else 'xs'[pos 'i'] > 4 {
            change 'kept' = 'kept' plus 1;
        }
        change 'i' = 'i' plus 1;
    }
    no_check my.exit[('hits' times 100) plus 'kept'];     # 109
}
