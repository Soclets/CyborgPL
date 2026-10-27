# Two regional indicators make one flag (UAX #29, GB12/GB13): "🇹🇭🇯🇵" is two
# graphemes, Thailand and Japan, and a lone indicator after them is its own.
!MAIN:
    let $f := "🇹🇭🇯🇵🇫";
    std::print.shortest(std::str.clusters(lend $f), " graphemes", newln);
    for $g in std::str.each.cluster(lend $f):
        std::print.shortest($g, "|");
    end of for
    std::print.shortest(newln);
!end of MAIN
