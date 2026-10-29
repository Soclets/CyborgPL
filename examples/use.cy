# `use`, one per line at the top of a file, Kotlin's shape (owner, 2026-10-22): the name after `as` stands for another
# kit's path in this file, and `as` is always written. It names a module or a type too, so `use std::ls as ls;`
# lets `ls[i64]` and `ls.append` stand for std's. A rename onto a path, `as std::exact`, is allowed (owner: "it's their
# choice"); every `use` gets a note in kayo's build report.
use std::ls as ls;
use std::print as say;
use std::print.stored as std::exact;

MAIN:
    var $xs: ls[i64] = [];
    for $i: i64 [from: 1, to: 4, by: 1]:
        ls.append(lend.mut $xs, value: $i x $i);
    ;for
    say(ls.len(lend $xs), " squares, the last ", $xs[pos: 4], newln);
    std::exact("0.1 as stored: ", 0.1, newln);
;MAIN
