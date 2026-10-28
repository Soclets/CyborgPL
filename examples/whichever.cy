# whichever, timers, structured tasks, a task that fails (design/concurrency.md, 2026-09-22).
fn fetch [lent $url: str.utf8] -> str.utf8 or fail[no_network]:
    if $url == "down.com":
        fail no_network;
    ;if
    ret "page of " ++ $url;
;fn fetch

MAIN:
    let $inbox: %channel[str.utf8][max: 10] := %channel.empty;
    let $shutdown: %channel[bool] := %channel.empty;
    %spawn.task with [lend $inbox]:
        %channel.send(lend $inbox, value: "first"); %channel.send(lend $inbox, value: "second");
    ;%spawn.task
    %spawn.task with [lend $shutdown]:
        %channel.send(lend $shutdown, value: true);
    ;%spawn.task
    let $deadline := std::time.after(5 secs);
    for $i [from: 1, to: 4, by: 1]:
        %whichever:
            case $msg from lend $inbox:
                std::println("got ", $msg);
            ;case $msg
            case $why from lend $shutdown:
                std::println("shutdown");
            ;case $why
            case $now from lend $deadline:
                std::println("timed out after ", $now);
            ;case $now
        ;%whichever
        otherwise:
            case closed:
                std::println("all closed");
            ;case closed
        ;otherwise;
    ;for
    block:
        %spawn.task:
            std::println("task in a block");
        ;%spawn.task
        std::println("block body");
    ;block
    std::println("after the block");
    %spawn.task with [lend $inbox]:
        let $p := fetch("down.com")?; %channel.send(lend $inbox, value: give $p);
    ;%spawn.task
        otherwise:
            case no_network:
                std::println("offline");
            ;case no_network
        ;otherwise;
    let $tick := std::time.every(1 sec);
    let $t := %channel.take(lend $tick) ?? 0 secs;
    std::println("tick ", $t);
;MAIN
