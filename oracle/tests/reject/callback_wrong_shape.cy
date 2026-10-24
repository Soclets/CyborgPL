extern.C fn qsort [$base: ptr.mut[i32], $n: u64, $size: u64, $cmp: fn[ptr[i32], ptr[i32]] -> i32];
fn one [$a: ptr[i32]] -> i32 { return 0; }
MAIN {
    let $nums := [3, 1, 2];
    no_check qsort(lend.mut $nums, 3, 4, one);
}
