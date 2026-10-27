fn f [] -> int.sign.32 or fail[a]:
    fail a;
end of fn f
!MAIN:
    let $x := f() plus 1 otherwise 0;
!end of MAIN
