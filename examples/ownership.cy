# Ownership: `lend`, `lend.mut`, `give` and `copy`.
fn count [lent $items: std::list[str.utf8]] -> i32 {
    let $n := 0;
    for $item in lend $items {
        change $n = $n + 1;
    }
    return $n;
}

fn shout [lent.mut $text: str.utf8] {
    change $text = copy $text concat "!";
}

fn greet [$name: str.utf8] -> str.utf8 {
    return "hello " concat $name;
}

MAIN {
    let $s := "line 1" concat newln concat "line 2";
    let $t := give $s;
    let $u := copy $t;
    shout(lend.mut $u);
    std::print.stored($t, newln);
    std::print.stored($u, newln);
    let $g := greet(copy $u);
    std::print.stored($g, newln);
    std::print.stored($u, newln);
    let $words := ["a", "b", "c"];
    let $n := count(lend $words);
    std::print.useful($n, " words", newln);
    let $m := count(["x", "y"]);
    std::print.useful($m, " fresh", newln);
    let $r := lend $words;
    std::print.useful($r, newln);
}
