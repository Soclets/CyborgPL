# Two regional indicators make one flag (UAX #29, GB12/GB13): "🇹🇭🇯🇵" is two
# graphemes, Thailand and Japan, and a lone indicator after them is its own.
MAIN {
    let 'f' := "🇹🇭🇯🇵🇫";
    std::print.useful[std::str.graphemes[lend 'f'], " graphemes", newln];
    for 'g' in std::str.each.grapheme[lend 'f'] {
        std::print.useful['g', "|"];
    }
    std::print.useful[newln];
}
