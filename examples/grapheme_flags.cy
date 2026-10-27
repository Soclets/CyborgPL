# Two regional indicators make one flag (UAX #29, GB12/GB13): "🇹🇭🇯🇵" is two
# graphemes, Thailand and Japan, and a lone indicator after them is its own.
!MAIN:
    let $f := "🇹🇭🇯🇵🇫";
    std::print.shortest(std::string.grapheme_clusters(lend $f), " graphemes", newline);
    for $g in std::string.each.grapheme_cluster(lend $f):
        std::print.shortest($g, "|");
    end of for
    std::print.shortest(newline);
!end of MAIN
