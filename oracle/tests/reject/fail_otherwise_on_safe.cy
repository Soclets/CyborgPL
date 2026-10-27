function f [] -> integer.signed.32_bit:
    return 1;
end of function f
!MAIN:
    let $x := f() otherwise 0;
!end of MAIN
