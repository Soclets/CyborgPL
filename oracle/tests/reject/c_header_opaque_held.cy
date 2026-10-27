incl.%C.system["stdio.h"];

fn f [let $x: FILE] -> int.sign.32:
    ret 1;
end of fn f

!MAIN:
    std::print.shortest("ran", newln);
!end of MAIN
