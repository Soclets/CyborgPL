%extern.C proc exit [$code: i32];
fn total [lent $xs: std::ls[i32]] -> i32:
    var $s: i32 = 0;
    for $x in lend $xs:
        change $s to $s + $x;
    ;for
    ret $s;
;fn total
MAIN:
    let $xs: std::ls[i32] := [1, 2, 3];
    let $name: str.utf8 := "task" concat "s";
    var $out: std::ls[i32] = [];
    for $k: i32 [from: 1, to: 50, by: 1]:
        %spawn.task with [copy $xs, give $name, lend.mut $out]:
            let $inner := total(lend $xs);
            block:
                %spawn.task with [lend $xs]:
                    let $again := total(lend $xs);
                ;%spawn.task
            ;block
            std::ls.append(lend.mut $out, value: $inner + std::str.bytes(lend $name));
        ;%spawn.task
        break;
    ;for
    no_check my.exit(std::ls.len(lend $out) + $out[pos: 1]);
;MAIN
