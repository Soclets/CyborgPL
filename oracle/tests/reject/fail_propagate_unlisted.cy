function f [] -> integer.signed.32_bit or fail[a]:
    fail a;
end of function f
function g [] -> integer.signed.32_bit or fail[b]:
    let $x := f() otherwise fail; return $x;
end of function g
!MAIN:
    let $y := g() otherwise 0;
!end of MAIN
