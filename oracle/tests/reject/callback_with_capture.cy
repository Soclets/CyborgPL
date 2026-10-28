%extern.C proc qsort [let $base: ptr.mut[i32], let $n: u64, let $size: u64, let $cmp: fn[ptr[i32], ptr[i32]] -> i32];
MAIN:
    var $nums = [3, 1, 2];
    let $flip := true;
    no_check qsort(lend.mut $nums, 3, 4, fn [let $a: ptr[i32], let $b: ptr[i32]] -> i32 with [lend $flip]:
        ret 0;
    ;fn);
;MAIN
