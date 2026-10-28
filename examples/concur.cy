# concur, channels, duration literals (design/concurrency.md, 2026-09-22).
fn fetch [lent $url: str.utf8] -> str.utf8:
    ret "page of " ++ $url;
;fn fetch

MAIN:
    # Each task sends on its own channel, so the pages print in the same order whichever finishes first.
    let $from_a: %channel[str.utf8] := %channel.empty;
    let $from_b: %channel[str.utf8] := %channel.empty;
    let $a := "a.com";
    %spawn.task with [lend $from_a, give $a]:
        %channel.send(lend $from_a, value: fetch(lend $a));
    ;%spawn.task
    %spawn.task with [lend $from_b]:
        %channel.send(lend $from_b, value: fetch("b.com"));
    ;%spawn.task
    let $page_a := %channel.take(lend $from_a) ?? str.empty;
    let $page_b := %channel.take(lend $from_b) ?? str.empty;
    std::print($page_a, newln, $page_b, newln);
    let $d := 5 secs;
    std::print($d, " ", 1500 millis, " ", 1 hour, " ", 5 secs == 5_000 millis, newln);
;MAIN
