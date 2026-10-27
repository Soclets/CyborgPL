function f [] -> integer.signed.32_bit or fail[oops]:
    fail oops;
end of function f
!MAIN:
    let $x := f();
!end of MAIN
