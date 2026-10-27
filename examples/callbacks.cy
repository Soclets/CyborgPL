# Callbacks (design/ffi.md, 2026-09-21): a capture-free, C-shaped CyborgPL function handed to C.
%external.C procedure qsort [$base: pointer.mutable[integer.signed.32_bit], $n: integer.unsigned.64_bit, $size: integer.unsigned.64_bit,
                 $cmp: function[pointer[integer.signed.32_bit], pointer[integer.signed.32_bit]] -> integer.signed.32_bit];

procedure ascending [$a: pointer[integer.signed.32_bit], $b: pointer[integer.signed.32_bit]] -> integer.signed.32_bit:
    !no_check return pointer.read($a) minus pointer.read($b);        # C hands pointers to two elements
end of procedure ascending

procedure descending [$a: pointer[integer.signed.32_bit], $b: pointer[integer.signed.32_bit]] -> integer.signed.32_bit:
    !no_check return pointer.read($b) minus pointer.read($a);
end of procedure descending

!MAIN:
    variable $nums := [5, 1, 4, 2, 3];
    !no_check qsort(lend.mutable $nums, 5, size.of[integer.signed.32_bit], ascending);
    std::print.shortest($nums, newline);
    !no_check qsort(lend.mutable $nums, 5, size.of[integer.signed.32_bit], descending);
    std::print.shortest($nums, newline);
!end of MAIN
