# Ownership: `lend`, `lend.mut`, `give` and `copy`.
function count [lent $items: std::list[string.utf8]] -> integer.signed.32_bit:
    variable $n := 0;
    for $item in lend $items:
        change $n = $n plus 1;
    end of for
    return $n;
end of function count

procedure shout [lent.mutable $text: string.utf8]:
    change $text = copy $text concatenate "!";
end of procedure shout

function greet [$name: string.utf8] -> string.utf8:
    return "hello " concatenate $name;
end of function greet

!MAIN:
    let $s := "line 1" concatenate newline concatenate "line 2";
    let $t := give $s;
    variable $u := copy $t;
    shout(lend.mutable $u);
    std::print.stored($t, newline);
    std::print.stored($u, newline);
    let $g := greet(copy $u);
    std::print.stored($g, newline);
    std::print.stored($u, newline);
    let $words := ["a", "b", "c"];
    let $n := count(lend $words);
    std::print.shortest($n, " words", newline);
    let $m := count(["x", "y"]);
    std::print.shortest($m, " fresh", newline);
    let $r := lend $words;
    std::print.shortest($r, newline);
!end of MAIN
