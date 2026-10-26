# Callbacks (design/ffi.md, 2026-09-21): a capture-free, C-shaped CyborgPL function handed to C.
%extern.C fn qsort [$base: ptr.mut[i32], $n: u64, $size: u64,
                 $cmp: fn[ptr[i32], ptr[i32]] -> i32];

fn ascending [$a: ptr[i32], $b: ptr[i32]] -> i32 {
    no_check return ptr.read($a) - ptr.read($b);        # C hands pointers to two elements
}

fn descending [$a: ptr[i32], $b: ptr[i32]] -> i32 {
    no_check return ptr.read($b) - ptr.read($a);
}

MAIN {
    let $nums := [5, 1, 4, 2, 3];
    no_check qsort(lend.mut $nums, 5, size.of[i32], ascending);
    std::print.useful($nums, newln);
    no_check qsort(lend.mut $nums, 5, size.of[i32], descending);
    std::print.useful($nums, newln);
}
