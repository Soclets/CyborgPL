# concur, channels, duration literals (design/concurrency.md, 2026-09-22).
fn fetch [lent $url: str.utf8] -> str.utf8:
    ret "page of " concat $url;
end of fn fetch

!MAIN:
    # Each task sends on its own channel, so the pages print in the same order whichever finishes first.
    let $from_a: %channel[str.utf8] = %channel.empty;
    let $from_b: %channel[str.utf8] = %channel.empty;
    let $a := "a.com";
    %spawn.task with [lend $from_a, give $a]:
        %channel.send(lend $from_a, value: fetch(lend $a));
    end of %spawn.task
    %spawn.task with [lend $from_b]:
        %channel.send(lend $from_b, value: fetch("b.com"));
    end of %spawn.task
    let $page_a := %channel.take(lend $from_a) otherwise str.empty;
    let $page_b := %channel.take(lend $from_b) otherwise str.empty;
    std::print.shortest($page_a, newln, $page_b, newln);
    let $d := 5 secs;
    std::print.shortest($d, " ", 1500 millis, " ", 1 hour, " ", 5 secs eq 5_000 millis, newln);
!end of MAIN
