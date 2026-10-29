%extern.C proc exit [$code: i32];
fn work [$n: i32] -> i32:
    ret $n x 2;
;fn work
proc fill [lent.mut $into: std::ls[i32], $from: i32]:
    for $i: i32 [from: 1, to: 5, by: 1]:
        std::ls.append(lend.mut $into, value: $from + $i);
    ;for
;proc fill
MAIN:
    var $a: std::ls[i32] = [];
    var $b: std::ls[i32] = [];
    block:
        %spawn.task with [lend.mut $a]:
            fill(lend.mut $a, 10);
        ;%spawn.task
        %spawn.task with [lend.mut $b]:
            fill(lend.mut $b, 20);
        ;%spawn.task
        %spawn.task work(4);
    ;block
    let $la: i32 := std::ls.len(lend $a);
    let $lb: i32 := std::ls.len(lend $b);
    let $last: i32 := $b[pos: 5];
    no_check my.exit($la + $lb + $last);
;MAIN
