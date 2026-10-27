# Callbacks (design/ffi.md, 2026-09-21): a capture-free, C-shaped CyborgPL function handed to C.
%extern.C proc qsort [$base: ptr.mut[int.sign.32], $n: int.unsign.64, $size: int.unsign.64,
                 $cmp: fn[ptr[int.sign.32], ptr[int.sign.32]] -> int.sign.32];

proc ascending [$a: ptr[int.sign.32], $b: ptr[int.sign.32]] -> int.sign.32:
    !no_check ret ptr.read($a) minus ptr.read($b);        # C hands pointers to two elements
end of proc ascending

proc descending [$a: ptr[int.sign.32], $b: ptr[int.sign.32]] -> int.sign.32:
    !no_check ret ptr.read($b) minus ptr.read($a);
end of proc descending

!MAIN:
    var $nums := [5, 1, 4, 2, 3];
    !no_check qsort(lend.mut $nums, 5, size.of[int.sign.32], ascending);
    std::print.shortest($nums, newln);
    !no_check qsort(lend.mut $nums, 5, size.of[int.sign.32], descending);
    std::print.shortest($nums, newln);
!end of MAIN
