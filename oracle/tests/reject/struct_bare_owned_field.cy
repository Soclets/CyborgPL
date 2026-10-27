record P:
    n: string.utf8;
end of record P
!MAIN:
    let $p := P[n = "a"];
    let $t := $p.n;
!end of MAIN
