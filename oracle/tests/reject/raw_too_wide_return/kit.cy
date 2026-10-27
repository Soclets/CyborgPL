function f [] -> integer.unsigned.16_bit:
    return 65536;
end of function f

!MAIN:
    let $x := f();
!end of MAIN
