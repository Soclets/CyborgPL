fn f [] -> int.sign.32:
    ret 1;
end of fn f
!MAIN:
    let $x := f() otherwise 0;
!end of MAIN
