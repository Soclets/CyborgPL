fn g [let $p: int.unsign.8] -> int.unsign.8:
    if false:
        ret true;
    end of if
    ret 1;
end of fn g
!MAIN:
    let $x := g(1);
!end of MAIN
