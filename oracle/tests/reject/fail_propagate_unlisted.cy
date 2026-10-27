fn f [] -> int.sign.32 or fail[a]:
    fail a;
end of fn f
fn g [] -> int.sign.32 or fail[b]:
    let $x := f() otherwise fail; ret $x;
end of fn g
!MAIN:
    let $y := g() otherwise 0;
!end of MAIN
