# Text in bytes, points and graphemes, and walking it one grapheme at a time.
MAIN:
    let $s := "héllo 👨‍👩‍👧 wörld";

    # a text has no single length — say which unit (decided 2026-09-20)
    std::println(std::str.bytes(lend $s), " bytes, ", std::str.points(lend $s), " points, ", std::str.clusters(lend $s), " graphemes");

    # walking a text says the unit too
    var $vowels = 0;
    for $g in std::str.each.cluster(lend $s):
        if $g == `o` | $g == `e`:
            change $vowels += 1;
        ;if
    ;for
    std::println($vowels, " vowels");

    # the toolbox is std's
    let $words := std::str.split(lend $s, at: " ");
    std::println(std::ls.len(lend $words), " words; first is ", $words[pos: 1]);
    std::println(std::str.upper.full(lend $s));
    std::println(std::str.contains(lend $s, part: "wör"), " ", std::str.starts_with(lend $s, part: "hé"));
    let $third := std::str.at.cluster(lend $s, pos: 3) ?? `?`;
    let $family := std::str.at.cluster(lend $s, pos: 7) ?? `?`;
    std::println($third, " ", $family);
    let $where := std::str.find.cluster(lend $s, part: "wör") ?? 0;
    std::println("wör starts at grapheme ", $where);
    let $head := std::str.slice.cluster(lend $s, from: pos 1, to: pos 5) ?? str.empty;
    std::println($head);
    std::println(std::str.join(lend $words, between: "_"));
;MAIN
