# concur, channels, duration literals (design/concurrency.md, 2026-09-22).
function fetch [lent $url: string.utf8] -> string.utf8:
    return "page of " concatenate $url;
end of function fetch

!MAIN:
    # Each task sends on its own channel, so the pages print in the same order whichever finishes first.
    let $from_a: %channel[string.utf8] = %channel.empty;
    let $from_b: %channel[string.utf8] = %channel.empty;
    let $a := "a.com";
    %spawn.task with [lend $from_a, give $a]:
        %channel.send(lend $from_a, value: fetch(lend $a));
    end of %spawn.task
    %spawn.task with [lend $from_b]:
        %channel.send(lend $from_b, value: fetch("b.com"));
    end of %spawn.task
    let $page_a := %channel.take(lend $from_a) otherwise string.empty;
    let $page_b := %channel.take(lend $from_b) otherwise string.empty;
    std::print.shortest($page_a, newline, $page_b, newline);
    let $d := 5 seconds;
    std::print.shortest($d, " ", 1500 milliseconds, " ", 1 hour, " ", 5 seconds equal_to 5_000 milliseconds, newline);
!end of MAIN
