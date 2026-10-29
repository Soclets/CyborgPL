%extern.C proc exit [$code: i32];
MAIN:
    let $c: %channel[i32] := %channel.empty;
    var $got: i32 = 0;
    block:
        %spawn.task with [lend $c]:
            %channel.send(lend $c, value: 5);
        ;%spawn.task
        %spawn.task with [lend $c, lend.mut $got]:
            let $x := %channel.take(lend $c) ?? 0;
            let $y := %channel.take(lend $c) ?? 100;
            change $got to $x + $y;
        ;%spawn.task
    ;block
    no_check my.exit($got);
;MAIN
