extern.C fn qsort ['base': ptr.mut[int32], 'n': uint64, 'size': uint64, 'cmp': fn[ptr[int32], ptr[int32]] -> int32];
MAIN {
    let 'nums' := [3, 1, 2];
    let 'flip' := true;
    no_check qsort[lend.mut 'nums', 3, 4, fn ['a': ptr[int32], 'b': ptr[int32]] -> int32 with [lend 'flip'] { return 0; }];
}
