# Ownership: `lend`, `lend.mut`, `give` and `copy`.
fn count [lent $items: std::ls[str.utf8]] -> int.sign.32:
    var $n := 0;
    for $item in lend $items:
        change $n = $n plus 1;
    end of for
    ret $n;
end of fn count

proc shout [lent.mut $text: str.utf8]:
    change $text = copy $text concat "!";
end of proc shout

fn greet [let $name: str.utf8] -> str.utf8:
    ret "hello " concat $name;
end of fn greet

!MAIN:
    let $s := "line 1" concat newln concat "line 2";
    let $t := give $s;
    var $u := copy $t;
    shout(lend.mut $u);
    std::print.stored($t, newln);
    std::print.stored($u, newln);
    let $g := greet(copy $u);
    std::print.stored($g, newln);
    std::print.stored($u, newln);
    let $words := ["a", "b", "c"];
    let $n := count(lend $words);
    std::print.shortest($n, " words", newln);
    let $m := count(["x", "y"]);
    std::print.shortest($m, " fresh", newln);
    let $r := lend $words;
    std::print.shortest($r, newln);
!end of MAIN
