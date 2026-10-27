# A case change says its mapping (owner, 2026-10-20): `.ascii` changes a to z only, `.simple` one character
# for one, `.full` is Unicode's, where `ß` becomes `SS`; `lang:` adds a language's own rules to the full one.
# A bare `str.upper` is refused naming the three.
!MAIN:
    let $s := "straße in istanbul, café";
    std::print.shortest(std::string.uppercase.ascii(lend $s), newline);
    std::print.shortest(std::string.uppercase.simple(lend $s), newline);
    std::print.shortest(std::string.uppercase.full(lend $s), newline);
    std::print.shortest(std::string.uppercase.full(lend $s, lang: std::casing.tr), newline);
    let $tag := "tr-TR";
    let $l := to.casing($tag);
    let $city := "ISTANBUL";
    std::print.shortest(std::string.lowercase.full(lend $city, lang: lend $l), " ", std::string.lowercase.full(lend $city, lang: std::casing.root), newline);
    let $shop := "CAFÉ";
    std::print.shortest(std::string.lowercase.ascii(lend $shop), " ", std::string.lowercase.full(lend $shop), newline);
!end of MAIN
