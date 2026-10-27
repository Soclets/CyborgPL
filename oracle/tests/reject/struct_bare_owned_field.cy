rec P:
    n: str.utf8;
end of rec P
!MAIN:
    let $p := P[n = "a"];
    let $t := $p.n;
!end of MAIN
