fn put [lent $c: %channel[i32]]:
    %channel.send(lend $c, value: 1);
;fn put
MAIN:
    let $c: %channel[i32][max: 1] := %channel.empty;
    put(lend $c);
    let $v := %channel.take(lend $c) ?? 0;
    std::print($v, newln);
;MAIN
