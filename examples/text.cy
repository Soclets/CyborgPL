# Text in bytes, points and graphemes, and walking it one grapheme at a time.
!MAIN:
    let $s := "héllo 👨‍👩‍👧 wörld";

    # a text has no single length — say which unit (decided 2026-09-20)
    std::print.shortest(std::string.bytes(lend $s), " bytes, ", std::string.code_points(lend $s), " points, ", std::string.grapheme_clusters(lend $s), " graphemes", newline);

    # walking a text says the unit too
    variable $vowels := 0;
    for $g in std::string.each.grapheme_cluster(lend $s):
        if $g equal_to `o` or $g equal_to `e`:
            change $vowels = $vowels plus 1;
        end of if
    end of for
    std::print.shortest($vowels, " vowels", newline);

    # the toolbox is std's
    let $words := std::string.split(lend $s, at: " ");
    std::print.shortest(std::list.length(lend $words), " words; first is ", $words[position: 1], newline);
    std::print.shortest(std::string.uppercase.full(lend $s), newline);
    std::print.shortest(std::string.contains(lend $s, part: "wör"), " ", std::string.starts_with(lend $s, part: "hé"), newline);
    let $third := std::string.at.grapheme_cluster(lend $s, position: 3) otherwise `?`;
    let $family := std::string.at.grapheme_cluster(lend $s, position: 7) otherwise `?`;
    std::print.shortest($third, " ", $family, newline);
    let $where := std::string.find.grapheme_cluster(lend $s, part: "wör") otherwise 0;
    std::print.shortest("wör starts at grapheme ", $where, newline);
    let $head := std::string.slice.grapheme_cluster(lend $s, from: position 1, to: position 5) otherwise string.empty;
    std::print.shortest($head, newline);
    std::print.shortest(std::string.join(lend $words, between: "_"), newline);
!end of MAIN
