# Ownership: `lend`, `lend.mut`, `give` and `copy`.
fn count [lent $items: std::ls[str.utf8]] -> i32:
    var $n = 0;
    for $item in lend $items:
        change $n += 1;
    ;for
    ret $n;
;fn count

proc shout [lent.mut $text: str.utf8]:
    change $text to copy $text ++ "!";
;proc shout

fn greet [let $name: str.utf8] -> str.utf8:
    ret "hello " ++ $name;
;fn greet

MAIN:
    let $s := "line 1" ++ newln ++ "line 2";
    let $t := give $s;
    var $u = copy $t;
    shout(lend.mut $u);
    std::println.stored($t);
    std::println.stored($u);
    let $g := greet(copy $u);
    std::println.stored($g);
    std::println.stored($u);
    let $words := ["a", "b", "c"];
    let $n := count(lend $words);
    std::println($n, " words");
    let $m := count(["x", "y"]);
    std::println($m, " fresh");
    let $r := lend $words;
    std::println($r);
;MAIN
