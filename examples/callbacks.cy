# Callbacks (design/ffi.md, 2026-09-21): a capture-free, C-shaped CyborgPL function handed to C.
%extern.C proc qsort [$base: ptr.mut[i32], $n: u64, $size: u64,
                 $cmp: proc[ptr[i32], ptr[i32]] -> i32];

proc ascending [$a: ptr[i32], $b: ptr[i32]] -> i32:
    no_check ret ptr.read($a) - ptr.read($b);        # C hands pointers to two elements
;proc ascending

proc descending [$a: ptr[i32], $b: ptr[i32]] -> i32:
    no_check ret ptr.read($b) - ptr.read($a);
;proc descending

MAIN:
    var $nums = [5, 1, 4, 2, 3];
    no_check qsort(lend.mut $nums, 5, size.of[i32], ascending);
    std::print($nums, newln);
    no_check qsort(lend.mut $nums, 5, size.of[i32], descending);
    std::print($nums, newln);
;MAIN
