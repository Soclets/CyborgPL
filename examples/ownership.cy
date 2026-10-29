# Ownership: `lend`, `lend.mut`, `give` and `copy`.
fn count [lent $items: std::ls[str.utf8]] -> i32:
    var $n = 0;
    for $item in lend $items:
        change $n to $n + 1;
    ;for
    ret $n;
;fn count

proc shout [lent.mut $text: str.utf8]:
    change $text to copy $text concat "!";
;proc shout

fn greet [$name: str.utf8] -> str.utf8:
    ret "hello " concat $name;
;fn greet

MAIN:
    let $s := "line 1" concat newln concat "line 2";
    let $t := give $s;
    var $u = copy $t;
    shout(lend.mut $u);
    std::print.stored($t, newln);
    std::print.stored($u, newln);
    let $g := greet(copy $u);
    std::print.stored($g, newln);
    std::print.stored($u, newln);
    let $words := ["a", "b", "c"];
    let $n := count(lend $words);
    std::print($n, " words", newln);
    let $m := count(["x", "y"]);
    std::print($m, " fresh", newln);
    let $r := lend $words;
    std::print($r, newln);
;MAIN
