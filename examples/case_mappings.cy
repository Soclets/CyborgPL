# A case change says its mapping (owner, 2026-10-20): `.ascii` changes a to z only, `.simple` one character
# for one, `.full` is Unicode's, where `ß` becomes `SS`; `lang:` adds a language's own rules to the full one.
# A bare `str.upper` is refused naming the three.
MAIN {
    let 's' := "straße in istanbul, café";
    std::print.useful[str.upper.ascii[lend 's'], newln];
    std::print.useful[str.upper.simple[lend 's'], newln];
    std::print.useful[str.upper.full[lend 's'], newln];
    std::print.useful[str.upper.full[lend 's', lang: tr], newln];
    let 'tag' := "tr-TR";
    let 'l' := to.casing['tag'];
    let 'city' := "ISTANBUL";
    std::print.useful[str.lower.full[lend 'city', lang: lend 'l'], " ", str.lower.full[lend 'city', lang: casing.root], newln];
    let 'shop' := "CAFÉ";
    std::print.useful[str.lower.ascii[lend 'shop'], " ", str.lower.full[lend 'shop'], newln];
}
