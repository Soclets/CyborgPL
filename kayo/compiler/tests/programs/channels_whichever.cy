%extern.C proc exit [$code: i32];
proc produce [lent $out: %channel[std::ls[i32]][min: 0, max: 4], $base: i32]:
    for $i: i32 [from: 1, to: 50, by: 1]:
        let $batch: std::ls[i32] := [$base, $i];
        %channel.send(lend $out, value: give $batch);
    ;for
;proc produce
MAIN:
    let $pipe: %channel[std::ls[i32]][min: 0, max: 4] := %channel.empty;
    let $quiet: %channel[i32] := %channel.empty;
    %spawn.task with [lend $pipe]:
        produce(lend $pipe, 1000);
    ;%spawn.task
    %spawn.task with [lend $pipe]:
        produce(lend $pipe, 2000);
    ;%spawn.task
    %spawn.task with [lend $quiet]:
        %channel.send(lend $quiet, value: 7);
    ;%spawn.task
    var $batches: i32 = 0;
    var $sevens: i32 = 0;
    var $closed: i32 = 0;
    for $k: i32 [from: 1, to: 102, by: 1]:
        %whichever:
            case $b from lend $pipe:
                change $batches to $batches + to.i32(std::ls.len(lend $b)) - 1;
            ;case $b
            case $s from lend $quiet:
                change $sevens to $sevens + $s;
            ;case $s
        ;%whichever
        otherwise:
            case closed:
                change $closed to $closed + 1;
            ;case closed
        ;otherwise;
    ;for
    no_check my.exit($batches + $sevens + $closed x 10);
;MAIN
