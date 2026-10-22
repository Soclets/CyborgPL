# A case change says its mapping (owner, 2026-10-20): `.ascii` changes a to z only, `.simple` one character
# for one, `.full` is Unicode's, where `ß` becomes `SS`; `lang:` adds a language's own rules to the full one.
# A bare `str.upper` is refused naming the three.
MAIN {
    let 's' := "straße in istanbul, café";
    std::print.useful[std::str.upper.ascii[lend 's'], newln];
    std::print.useful[std::str.upper.simple[lend 's'], newln];
    std::print.useful[std::str.upper.full[lend 's'], newln];
    std::print.useful[std::str.upper.full[lend 's', lang: std::casing.tr], newln];
    let 'tag' := "tr-TR";
    let 'l' := to.casing['tag'];
    let 'city' := "ISTANBUL";
    std::print.useful[std::str.lower.full[lend 'city', lang: lend 'l'], " ", std::str.lower.full[lend 'city', lang: std::casing.root], newln];
    let 'shop' := "CAFÉ";
    std::print.useful[std::str.lower.ascii[lend 'shop'], " ", std::str.lower.full[lend 'shop'], newln];
}
