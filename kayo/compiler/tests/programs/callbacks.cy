# A capture-free, C-shaped Cyborg function handed to C's qsort (ffi.md): the
# callback crosses as its code, C calls it with C's convention, and the
# list lent to C's pointer takes the pointer's width (a `:=` of bare numbers).
extern.C fn exit [$code: i32];
extern.C fn qsort [$base: ptr.mut[i32], $n: u64, $size: u64,
                 $cmp: fn[ptr[i32], ptr[i32]] -> i32];
fn ascending [$a: ptr[i32], $b: ptr[i32]] -> i32 {
    no_check return ptr.read($a) minus ptr.read($b);
}
fn descending [$a: ptr[i32], $b: ptr[i32]] -> i32 {
    no_check return ptr.read($b) minus ptr.read($a);
}
MAIN {
    let mut $nums := [5, 1, 4, 2, 3];
    let mut $got: i32 = 0;
    no_check qsort(lend.mut $nums, 5, size.of[i32], ascending);
    if $nums[pos: 1] equal_to 1 and $nums[pos: 5] equal_to 5 and $nums[pos: 3] equal_to 3 { change $got = $got plus 1; }
    let $f := descending;
    no_check qsort(lend.mut $nums, 5, size.of[i32], $f);
    if $nums[pos: 1] equal_to 5 and $nums[pos: 2] equal_to 4 and $nums[pos: 5] equal_to 1 { change $got = $got plus 2; }
    no_check my.exit($got);
}
