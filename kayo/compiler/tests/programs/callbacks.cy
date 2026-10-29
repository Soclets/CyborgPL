# A capture-free, C-shaped Cyborg function handed to C's qsort (ffi.md): the
# callback crosses as its code, C calls it with C's convention, and the
# list lent to C's pointer takes the pointer's width (a `:=` of bare numbers).
%extern.C proc exit [let $code: i32];
%extern.C proc qsort [let $base: ptr.mut[i32], let $n: u64, let $size: u64,
                 let $cmp: proc[ptr[i32], ptr[i32]] -> i32];
proc ascending [let $a: ptr[i32], let $b: ptr[i32]] -> i32:
    no_check ret ptr.read($a) - ptr.read($b);
;proc ascending
proc descending [let $a: ptr[i32], let $b: ptr[i32]] -> i32:
    no_check ret ptr.read($b) - ptr.read($a);
;proc descending
MAIN:
    var $nums = [5, 1, 4, 2, 3];
    var $got: i32 = 0;
    no_check qsort(lend.mut $nums, 5, size.of[i32], ascending);
    if $nums[pos: 1] == 1 and $nums[pos: 5] == 5 and $nums[pos: 3] == 3:
        change $got to $got + 1;
    ;if
    let $f := descending;
    no_check qsort(lend.mut $nums, 5, size.of[i32], $f);
    if $nums[pos: 1] == 5 and $nums[pos: 2] == 4 and $nums[pos: 5] == 1:
        change $got to $got + 2;
    ;if
    no_check my.exit($got);
;MAIN
