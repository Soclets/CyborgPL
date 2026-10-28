%extern.C proc qsort [let $base: ptr.mut[i32], let $n: u64, let $size: u64, let $cmp: fn[ptr[i32], ptr[i32]] -> i32];
fn one [let $a: ptr[i32]] -> i32:
    ret 0;
;fn one
MAIN:
    var $nums = [3, 1, 2];
    no_check qsort(lend.mut $nums, 3, 4, one);
;MAIN
