module a:
    rec P:
        x: int.sign.32;
    end of rec P
end of module a
!MAIN:
    let $p := a.P[x = 1];
!end of MAIN
