# whichever, timers, structured tasks, a task that fails (design/concurrency.md, 2026-09-22).
fn fetch [lent $url: str.utf8] -> str.utf8 or fail[no_network] {
    if $url equal_to "down.com" { fail no_network; }
    return "page of " concat $url;
}

MAIN {
    let $inbox: channel[str.utf8][max: 10] = channel.empty;
    let $shutdown: channel[bool] = channel.empty;
    concur with [lend $inbox] { channel.send(lend $inbox, value: "first"); channel.send(lend $inbox, value: "second"); }
    concur with [lend $shutdown] { channel.send(lend $shutdown, value: true); }
    let $deadline := std::time.after(5 secs);
    for $i [from: 1, to: 4, by: 1] {
        whichever {
            $msg from lend $inbox    { std::print.useful("got ", $msg, newln); }
            $why from lend $shutdown { std::print.useful("shutdown", newln); }
            $now from lend $deadline { std::print.useful("timed out after ", $now, newln); }
        } otherwise { closed { std::print.useful("all closed", newln); } };
    }
    {
        concur { std::print.useful("task in a block", newln); }
        std::print.useful("block body", newln);
    }
    std::print.useful("after the block", newln);
    concur with [lend $inbox] { let $p := fetch("down.com") otherwise fail; channel.send(lend $inbox, value: give $p); }
        otherwise { no_network { std::print.useful("offline", newln); } };
    let $tick := std::time.every(1 sec);
    let $t := channel.take(lend $tick) otherwise 0 secs;
    std::print.useful("tick ", $t, newln);
}
