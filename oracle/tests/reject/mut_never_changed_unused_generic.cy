fn keep <T: any> [let $v: <T>] -> <T>:
    var $x = $v;
    ret $x;
end of fn keep

!MAIN:
    std::print.shortest("ok", newln);
!end of MAIN
