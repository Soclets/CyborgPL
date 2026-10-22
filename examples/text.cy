# Text in bytes, points and graphemes, and walking it one grapheme at a time.
MAIN {
    let 's' := "héllo 👨‍👩‍👧 wörld";

    # a text has no single length — say which unit (decided 2026-09-20)
    std::print.useful[std::str.bytes[lend 's'], " bytes, ", std::str.points[lend 's'], " points, ", std::str.graphemes[lend 's'], " graphemes", newln];

    # walking a text says the unit too
    let 'vowels' := 0;
    for 'g' in std::str.each.grapheme[lend 's'] {
        if 'g' equal_to `o` or 'g' equal_to `e` { change 'vowels' = 'vowels' + 1; }
    }
    std::print.useful['vowels', " vowels", newln];

    # the toolbox is std's
    let 'words' := std::str.split[lend 's', at: " "];
    std::print.useful[std::list.length[lend 'words'], " words; first is ", 'words'[pos: 1], newln];
    std::print.useful[std::str.upper.full[lend 's'], newln];
    std::print.useful[std::str.contains[lend 's', part: "wör"], " ", std::str.starts_with[lend 's', part: "hé"], newln];
    let 'third' := std::str.at.grapheme[lend 's', pos: 3] otherwise `?`;
    let 'family' := std::str.at.grapheme[lend 's', pos: 7] otherwise `?`;
    std::print.useful['third', " ", 'family', newln];
    let 'where' := std::str.find.grapheme[lend 's', part: "wör"] otherwise 0;
    std::print.useful["wör starts at grapheme ", 'where', newln];
    let 'head' := std::str.slice.grapheme[lend 's', from: pos 1, to: pos 5] otherwise str.empty;
    std::print.useful['head', newln];
    std::print.useful[std::str.join[lend 'words', between: "_"], newln];
}
