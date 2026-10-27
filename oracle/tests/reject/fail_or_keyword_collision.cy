function f [] -> integer.signed.32_bit or fail[a]:
    fail a;
end of function f
!MAIN:
    let $x := f() or 0;
!end of MAIN
