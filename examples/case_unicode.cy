# Insensitive means Unicode's full case folding (owner, 2026-10-06: "`b`"): `ß` matches `SS`, `É`
# matches `é`. Names beyond English come from WSTN, which a file with no CyJet.cyr has.
case.any [function, variable];

function Größe [] -> integer.signed.32_bit:
    return 7;
end of function Größe

!MAIN:
    let $Café := GRÖSSE();
    std::print.shortest(größe(), " ", $CAFÉ, " ", $café, newline);
!end of MAIN
