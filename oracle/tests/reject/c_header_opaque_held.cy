include.%C.system["stdio.h"];

function f [$x: FILE] -> integer.signed.32_bit:
    return 1;
end of function f

!MAIN:
    std::print.shortest("ran", newline);
!end of MAIN
