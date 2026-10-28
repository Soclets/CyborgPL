# Callbacks (design/ffi.md, 2026-09-21): a capture-free, C-shaped CyborgPL function handed to C.
%extern.C proc qsort [let $base: ptr.mut[i32], let $n: u64, let $size: u64,
                 let $cmp: proc[ptr[i32], ptr[i32]] -> i32];

proc ascending [let $a: ptr[i32], let $b: ptr[i32]] -> i32:
    no_check ret ptr.read($a) - ptr.read($b);        # C hands pointers to two elements
;proc ascending

proc descending [let $a: ptr[i32], let $b: ptr[i32]] -> i32:
    no_check ret ptr.read($b) - ptr.read($a);
;proc descending

MAIN:
    var $nums = [5, 1, 4, 2, 3];
    no_check qsort(lend.mut $nums, 5, size.of[i32], ascending);
    std::println($nums);
    no_check qsort(lend.mut $nums, 5, size.of[i32], descending);
    std::println($nums);
;MAIN
