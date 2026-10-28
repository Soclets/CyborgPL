# Symbols (owner, 2026-10-16: "only if it reads correctly"): each spells the plain form of its word, and
# writing them is good practice; a variation is the word (`plus.wrap`, `div.floor`, `neg.ieee`).
#   + -   plus, minus      x   times      == != < > <= >=   the comparisons     power is `std::pow`
#   -5, -$x   a negative, negation
# Everything else is a word: div.…, mod.…, and, or, and.then, or.else, not, concat, xor, shl, shr,
# lend / lent, ptr[T], exactly; and a change is always written out, `change $i to $i + 1;`.
MAIN:
    let $a := 7;
    let $b := 2;
    std::print($a + $b, " ", $a - $b, " ", $a x $b, " ", $b std::pow 10, " ", 1 + 2 x 3, newln);
    std::print($a < $b, " ", $a > $b, " ", $a < $b, " ", $a > $b, newln);
    std::print(-$a, " ", -$a, " ", -5, newln);
    std::print($a + $b, " ", $a div.floor $b, " ", $a == $b, newln);
    var $i = 1;
    change $i to $i + 1;
    std::print($i, newln);
;MAIN
