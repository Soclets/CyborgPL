%extern.C proc qsort [$base: ptr.mut[i32], $n: u64, $size: u64, $cmp: fn[ptr[i32], ptr[i32]] -> i32];
MAIN:
    var $nums = [3, 1, 2];
    let $flip := true;
    no_check qsort(lend.mut $nums, 3, 4, fn [$a: ptr[i32], $b: ptr[i32]] -> i32 with [lend $flip]:
        ret 0;
    ;fn);
;MAIN
