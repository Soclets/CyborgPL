# `shout` is case-insensitive (`case.any [fn, struct, field, const];` in its `kit.cy`), so
# this kit reaches its names in any case, with the path and bare (owner, 2026-10-06).
MAIN {
    print.useful[shout::SHOUT["hey"], " ", WHISPER[], " ", shout::'VOLUME', newln];
    let 'l' := shout::LEVEL[LOUD = true];
    print.useful['l'.loud, newln];
}
