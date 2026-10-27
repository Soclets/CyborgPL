function f [] -> integer.signed.32_bit or fail[a, b]:
    fail a;
end of function f
!MAIN:
    let $x := f() otherwise:
        !case a:
            return;
        !end of case a
    end of otherwise;
!end of MAIN
