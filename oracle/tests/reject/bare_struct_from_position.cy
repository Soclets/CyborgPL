%layout.C rec Pr:
    var a: u32;
    var b: u32;
;%layout.C rec Pr
MAIN:
    let $prs: std::ls[Pr] := [Pr[a = 1, b = 2],];
    let $p := $prs[pos: 1];
;MAIN
