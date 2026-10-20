extern.C fn qsort ['base': ptr.mut[i32], 'n': u64, 'size': u64, 'cmp': fn[ptr[i32], ptr[i32]] -> i32];
MAIN {
    let 'nums' := [3, 1, 2];
    let 'flip' := true;
    no_check qsort[lend.mut 'nums', 3, 4, fn ['a': ptr[i32], 'b': ptr[i32]] -> i32 with [lend 'flip'] { return 0; }];
}
