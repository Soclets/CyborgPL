# Symbols (owner, 2026-10-16: "only if it reads correctly"): each spells the plain form of its word, and
# writing them is good practice; a variation is the word (`plus.wrap`, `div.floor`, `negate.ieee`).
#   + -   plus, minus        x   times        xx   std's pow        < >   less_than, greater_than
#   -5, -'x'   a negative, negate 'x'
# Everything else is a word: equal_to, not_equal, at_most, at_least, div.…, mod.…, and, and.then, or.else,
# concat, lend / lent, ptr[T], exactly; and a change is always written out, `change $i = $i + 1;`.
MAIN {
    let $a := 7;
    let $b := 2;
    std::print.useful($a + $b, " ", $a - $b, " ", $a x $b, " ", $b xx 10, " ", 1 + 2 x 3, newln);
    std::print.useful($a < $b, " ", $a > $b, " ", $a less_than $b, " ", $a greater_than $b, newln);
    std::print.useful(-$a, " ", negate $a, " ", -5, newln);
    std::print.useful($a + $b, " ", $a div.floor $b, " ", $a equal_to $b, newln);
    let mut $i := 1;
    change $i = $i + 1;
    std::print.useful($i, newln);
}
