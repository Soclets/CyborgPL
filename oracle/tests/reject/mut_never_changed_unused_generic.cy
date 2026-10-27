function keep <T: any> [$v: <T>] -> <T>:
    variable $x := $v;
    return $x;
end of function keep

!MAIN:
    std::print.shortest("ok", newline);
!end of MAIN
