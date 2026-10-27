# `<-` feeds `print.weird` one value at a time. The arrows are short (owner, 2026-10-07: "Short,
# the neck & eye movement is less"); the long `<--`, main from 2026-10-03, is now refused.
fn double [$n: int.sign.32] -> int.sign.32:
    ret $n times 2;
end of fn double
!MAIN:
    std::print.arrows.shortest <- double(3) <- newln;
!end of MAIN
