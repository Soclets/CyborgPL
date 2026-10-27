# Text in bytes, points and graphemes, and walking it one grapheme at a time.
!MAIN:
    let $s := "héllo 👨‍👩‍👧 wörld";

    # a text has no single length — say which unit (decided 2026-09-20)
    std::print.shortest(std::str.bytes(lend $s), " bytes, ", std::str.points(lend $s), " points, ", std::str.clusters(lend $s), " graphemes", newln);

    # walking a text says the unit too
    var $vowels := 0;
    for $g in std::str.each.cluster(lend $s):
        if $g eq `o` or $g eq `e`:
            change $vowels to $vowels plus 1;
        end of if
    end of for
    std::print.shortest($vowels, " vowels", newln);

    # the toolbox is std's
    let $words := std::str.split(lend $s, at: " ");
    std::print.shortest(std::ls.len(lend $words), " words; first is ", $words[pos: 1], newln);
    std::print.shortest(std::str.upper.full(lend $s), newln);
    std::print.shortest(std::str.contains(lend $s, part: "wör"), " ", std::str.starts_with(lend $s, part: "hé"), newln);
    let $third := std::str.at.cluster(lend $s, pos: 3) otherwise `?`;
    let $family := std::str.at.cluster(lend $s, pos: 7) otherwise `?`;
    std::print.shortest($third, " ", $family, newln);
    let $where := std::str.find.cluster(lend $s, part: "wör") otherwise 0;
    std::print.shortest("wör starts at grapheme ", $where, newln);
    let $head := std::str.slice.cluster(lend $s, from: pos 1, to: pos 5) otherwise str.empty;
    std::print.shortest($head, newln);
    std::print.shortest(std::str.join(lend $words, between: "_"), newln);
!end of MAIN
