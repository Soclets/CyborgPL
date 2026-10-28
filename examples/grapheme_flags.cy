# Two regional indicators make one flag (UAX #29, GB12/GB13): "🇹🇭🇯🇵" is two
# graphemes, Thailand and Japan, and a lone indicator after them is its own.
MAIN:
    let $f := "🇹🇭🇯🇵🇫";
    std::print(std::str.clusters(lend $f), " graphemes", newln);
    for $g in std::str.each.cluster(lend $f):
        std::print($g, "|");
    ;for
    std::print(newln);
;MAIN
