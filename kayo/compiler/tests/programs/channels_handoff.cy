%extern.C proc exit [$code: i32];
fn fetch [lent $url: str.utf8] -> str.utf8:
    ret "page of " concat $url;
;fn fetch
MAIN:
    let $results: %channel[str.utf8] := %channel.empty;
    let $a := "a.com";
    %spawn.task with [lend $results, give $a]:
        %channel.send(lend $results, value: fetch(lend $a));
    ;%spawn.task
    %spawn.task with [lend $results]:
        %channel.send(lend $results, value: fetch("b.com"));
    ;%spawn.task
    var $total: i32 = 0;
    for $i [from: 1, to: 2, by: 1]:
        let $page := %channel.take(lend $results) ?? str.empty;
        change $total to $total + std::str.bytes(lend $page);
    ;for
    let $extra := %channel.take(lend $results) ?? "closed!";
    no_check my.exit($total + std::str.bytes(lend $extra));
;MAIN
