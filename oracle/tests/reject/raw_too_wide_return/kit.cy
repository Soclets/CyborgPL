fn f [] -> int.unsign.16:
    ret 65536;
end of fn f

!MAIN:
    let $x := f();
!end of MAIN
