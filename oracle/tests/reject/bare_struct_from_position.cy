%layout.C rec Pr:
    var a: int.unsign.32;
    var b: int.unsign.32;
end of rec Pr
!MAIN:
    let $prs: std::ls[Pr] = [Pr[a = 1, b = 2],];
    let $p := $prs[pos: 1];
!end of MAIN
