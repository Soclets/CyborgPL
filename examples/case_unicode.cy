# Insensitive means Unicode's full case folding (owner, 2026-10-06: "`b`"): `ß` matches `SS`, `É`
# matches `é`. Names beyond English come from WSTN, which a file with no CyJet.cyr has.
case.any [fn, var];

fn Größe [] -> int32 {
    return 7;
}

MAIN {
    let 'Café' := GRÖSSE[];
    std::print.useful[größe[], " ", 'CAFÉ', " ", 'café', newln];
}
