# whichever, timers, structured tasks, a task that fails (design/concurrency.md, 2026-09-22).
fn fetch [lent $url: str.utf8] -> str.utf8 or fail[no_network]:
    if $url eq "down.com":
        fail no_network;
    end of if
    ret "page of " concat $url;
end of fn fetch

!MAIN:
    let $inbox: %channel[str.utf8][max: 10] = %channel.empty;
    let $shutdown: %channel[bool] = %channel.empty;
    %spawn.task with [lend $inbox]:
        %channel.send(lend $inbox, value: "first"); %channel.send(lend $inbox, value: "second");
    end of %spawn.task
    %spawn.task with [lend $shutdown]:
        %channel.send(lend $shutdown, value: true);
    end of %spawn.task
    let $deadline := std::time.after(5 secs);
    for $i [from: 1, to: 4, by: 1]:
        %whichever:
            case $msg from lend $inbox:
                std::print.shortest("got ", $msg, newln);
            end of case $msg
            case $why from lend $shutdown:
                std::print.shortest("shutdown", newln);
            end of case $why
            case $now from lend $deadline:
                std::print.shortest("timed out after ", $now, newln);
            end of case $now
        end of %whichever
        otherwise:
            case closed:
                std::print.shortest("all closed", newln);
            end of case closed
        end of otherwise;
    end of for
    block:
        %spawn.task:
            std::print.shortest("task in a block", newln);
        end of %spawn.task
        std::print.shortest("block body", newln);
    end of block
    std::print.shortest("after the block", newln);
    %spawn.task with [lend $inbox]:
        let $p := fetch("down.com") otherwise fail; %channel.send(lend $inbox, value: give $p);
    end of %spawn.task
        otherwise:
            case no_network:
                std::print.shortest("offline", newln);
            end of case no_network
        end of otherwise;
    let $tick := std::time.every(1 sec);
    let $t := %channel.take(lend $tick) otherwise 0 secs;
    std::print.shortest("tick ", $t, newln);
!end of MAIN
