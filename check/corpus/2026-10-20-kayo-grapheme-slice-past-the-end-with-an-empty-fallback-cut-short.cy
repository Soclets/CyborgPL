# Finding 2026-10-20 (check run, seed 14027, cut down by hand): a grapheme slice past the end of a text
# whose graphemes are fewer than its code points (two flags: 3 graphemes, 5 points) fails as it should
# with `otherwise "-"`, but with `otherwise str.empty` kayo hands back the slice cut short: from: pos 2,
# to: pos 4 of "🇹🇭🇯🇵!" gives 🇯🇵! (9 bytes), where the oracle gives the empty text (0). In MAIN and in
# a function alike, tables on or off. Plain "hello" agrees both ways. kayo with std/ (both phases).
fn g [] -> u64 {
    let 't20': str.utf8 = "🇹🇭🇯🇵!";
    let 't21' := std::str.slice.grapheme[lend 't20', from: pos 2, to: pos 4] otherwise str.empty;
    return str.bytes[lend 't21'];
}
MAIN {
    let 's': str.utf8 = "🇹🇭🇯🇵!";
    let 'a' := std::str.slice.grapheme[lend 's', from: pos 2, to: pos 4] otherwise str.empty;
    std::print.useful[str.bytes[lend 'a'], newln];
    std::print.useful[g[], newln];
}
