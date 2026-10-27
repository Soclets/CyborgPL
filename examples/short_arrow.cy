# `<-` feeds `print.weird` one value at a time. The arrows are short (owner, 2026-10-07: "Short,
# the neck & eye movement is less"); the long `<--`, main from 2026-10-03, is now refused.
function double [$n: integer.signed.32_bit] -> integer.signed.32_bit:
    return $n times 2;
end of function double
!MAIN:
    std::print.arrows.shortest <- double(3) <- newline;
!end of MAIN
