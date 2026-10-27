fn half_start <T: any> [let $x: <T>] -> <T>:
    let $h: <T> := 0.5;
    ret $h;
end of fn half_start
!MAIN:
    std::print.shortest(half_start(4: int.sign.32), newln);
!end of MAIN
