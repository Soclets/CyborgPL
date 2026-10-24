# a returned borrow must be in the group it names (owner, 2026-09-29)
fn wrong [@x, @y] [lent @x $a: str.utf8, lent @y $b: str.utf8] -> lent @x str.utf8 { return lend $b; }
MAIN { let $s := "a"; let $t := "b"; let $r := wrong(lend $s, lend $t); }
