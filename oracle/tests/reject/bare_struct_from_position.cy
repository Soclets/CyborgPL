%layout.C record Pr:
    a: integer.unsigned.32_bit;
    b: integer.unsigned.32_bit;
end of record Pr
!MAIN:
    let $prs: std::list[Pr] = [Pr[a = 1, b = 2],];
    let $p := $prs[position: 1];
!end of MAIN
