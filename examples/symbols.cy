# Symbols (owner, 2026-10-16: "only if it reads correctly"): each spells the plain form of its word, and
# writing them is good practice; a variation is the word (`plus.wrap`, `div.floor`, `negate.ieee`).
#   + -   plus, minus        x   times        xx   std's pow        < >   less_than, greater_than
#   -5, -'x'   a negative, negate 'x'
# Everything else is a word: equal_to, not_equal, at_most, at_least, div.…, mod.…, and, and.then, or.else,
# concat, lend / lent, ptr[T], exactly; and a change is always written out, `change $i = $i + 1;`.
!MAIN:
    let $a := 7;
    let $b := 2;
    std::print.shortest($a plus $b, " ", $a minus $b, " ", $a times $b, " ", $b power 10, " ", 1 plus 2 times 3, newline);
    std::print.shortest($a less_than $b, " ", $a greater_than $b, " ", $a less_than $b, " ", $a greater_than $b, newline);
    std::print.shortest(negate $a, " ", negate $a, " ", negative 5, newline);
    std::print.shortest($a plus $b, " ", $a divide.floor $b, " ", $a equal_to $b, newline);
    variable $i := 1;
    change $i = $i plus 1;
    std::print.shortest($i, newline);
!end of MAIN
