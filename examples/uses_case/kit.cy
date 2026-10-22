# `shout` is case-insensitive (`case.any [fn, struct, field, const];` in its `kit.cy`), so
# this kit reaches its names in any case, always with the path (owner, 2026-10-06; the path, 2026-10-22).
MAIN {
    std::print.useful[shout::SHOUT["hey"], " ", shout::WHISPER[], " ", shout::'VOLUME', newln];
    let 'l' := shout::LEVEL[LOUD = true];
    std::print.useful['l'.loud, newln];
}
