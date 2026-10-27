# `use`, one per line at the top of a file, Kotlin's shape (owner, 2026-10-22): the name after `as` stands for another
# kit's path in this file, and `as` is always written. It names a module or a type too, so `use std::list as list;`
# brings back `list[i64]` and `list.append`. A rename onto a path, `as std::print`, is allowed (owner: "it's their
# choice"); every `use` gets a note in kayo's build report.
use std::ls as ls;
use std::print.shortest as say;
use std::print.stored as std::print;

!MAIN:
    var $xs: ls[int.sign.64] := [];
    for $i: int.sign.64 [from: 1, to: 4, by: 1]:
        ls.append(lend.mut $xs, value: $i times $i);
    end of for
    say(ls.len(lend $xs), " squares, the last ", $xs[pos: 4], newln);
    std::print("0.1 as stored: ", 0.1, newln);
!end of MAIN
