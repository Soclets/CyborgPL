function f [] -> integer.signed.32_bit or fail[a]:
    fail a;
end of function f
!MAIN:
    let $x := f() plus 1 otherwise 0;
!end of MAIN
