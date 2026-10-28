# whichever, timers, structured tasks, a task that fails (design/concurrency.md, 2026-09-22).
fn fetch [lent $url: str.utf8] -> str.utf8 or fail[no_network]:
    if $url == "down.com":
        fail no_network;
    ;if
    ret "page of " concat $url;
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
                std::print("got ", $msg, newln);
            ;case $msg
            case $why from lend $shutdown:
                std::print("shutdown", newln);
            ;case $why
            case $now from lend $deadline:
                std::print("timed out after ", $now, newln);
            ;case $now
        ;%whichever
        otherwise:
            case closed:
                std::print("all closed", newln);
            ;case closed
        ;otherwise;
    ;for
    block:
        %spawn.task:
            std::print("task in a block", newln);
        ;%spawn.task
        std::print("block body", newln);
    ;block
    std::print("after the block", newln);
    %spawn.task with [lend $inbox]:
        let $p := fetch("down.com")?; %channel.send(lend $inbox, value: give $p);
    ;%spawn.task
        otherwise:
            case no_network:
                std::print("offline", newln);
            ;case no_network
        ;otherwise;
    let $tick := std::time.every(1 sec);
    let $t := %channel.take(lend $tick) ?? 0 secs;
    std::print("tick ", $t, newln);
;MAIN
