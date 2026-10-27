# whichever, timers, structured tasks, a task that fails (design/concurrency.md, 2026-09-22).
function fetch [lent $url: string.utf8] -> string.utf8 or fail[no_network]:
    if $url equal_to "down.com":
        fail no_network;
    end of if
    return "page of " concatenate $url;
end of function fetch

!MAIN:
    let $inbox: %channel[string.utf8][maximum: 10] = %channel.empty;
    let $shutdown: %channel[boolean] = %channel.empty;
    %spawn.task with [lend $inbox]:
        %channel.send(lend $inbox, value: "first"); %channel.send(lend $inbox, value: "second");
    end of %spawn.task
    %spawn.task with [lend $shutdown]:
        %channel.send(lend $shutdown, value: true);
    end of %spawn.task
    let $deadline := std::time.after(5 seconds);
    for $i [from: 1, to: 4, by: 1]:
        %whichever:
            case $msg from lend $inbox:
                std::print.shortest("got ", $msg, newline);
            end of case $msg
            case $why from lend $shutdown:
                std::print.shortest("shutdown", newline);
            end of case $why
            case $now from lend $deadline:
                std::print.shortest("timed out after ", $now, newline);
            end of case $now
        end of %whichever
        otherwise:
            case closed:
                std::print.shortest("all closed", newline);
            end of case closed
        end of otherwise;
    end of for
    block:
        %spawn.task:
            std::print.shortest("task in a block", newline);
        end of %spawn.task
        std::print.shortest("block body", newline);
    end of block
    std::print.shortest("after the block", newline);
    %spawn.task with [lend $inbox]:
        let $p := fetch("down.com") otherwise fail; %channel.send(lend $inbox, value: give $p);
    end of %spawn.task
        otherwise:
            case no_network:
                std::print.shortest("offline", newline);
            end of case no_network
        end of otherwise;
    let $tick := std::time.every(1 second);
    let $t := %channel.take(lend $tick) otherwise 0 seconds;
    std::print.shortest("tick ", $t, newline);
!end of MAIN
