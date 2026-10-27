# Insensitive means Unicode's full case folding (owner, 2026-10-06: "`b`"): `ß` matches `SS`, `É`
# matches `é`. Names beyond English come from WSTN, which a file with no CyJet.cyr has.
case.any [fn, var];

fn Größe [] -> int.sign.32:
    ret 7;
end of fn Größe

!MAIN:
    let $Café := GRÖSSE();
    std::print.shortest(größe(), " ", $CAFÉ, " ", $café, newln);
!end of MAIN
