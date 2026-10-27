function f [] -> fail[a]:
    fail a;
end of function f
!MAIN:
    f() otherwise;
!end of MAIN
