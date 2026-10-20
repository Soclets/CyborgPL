# Counting loops: `for 'i' from 1 to 3 by 1`, up, down and by steps.
MAIN {
    for 'i' from 1 to 3 by 1 {
        std::print.useful['i', " "];
    }
    std::print.stored[newln];
    for 'k' from 0 to 10 by 5 {
        std::print.useful['k', " "];
    }
    std::print.stored[newln];
    for 'd' from 3 to 1 by -1 {
        std::print.useful['d', " "];
    }
    std::print.stored[newln];
    for perm 'stop' from 0 to 100 by 1 {
        if ('stop' x 'stop') > 50 { break; }
    }
    std::print.useful["stopped at ", 'stop', newln];
    for perm 'never' from 5 to 2 by 1 {
        std::print.stored["unreachable", newln];
    }
    std::print.useful["never ran, holds ", 'never', newln];
    for perm 'done' from 0 to 3 by 1 {
    }
    std::print.useful["ran out, holds ", 'done', newln];
    for 'b': i8 from 126 to 127 by 1 {
        std::print.useful['b', " "];
    }
    std::print.stored[newln];
    let 'words' := ["a", "b", "c"];
    for perm 'w' in lend 'words' {
    }
    std::print.stored["last word ", 'w', newln];
}
