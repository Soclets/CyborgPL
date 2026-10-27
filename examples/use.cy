# `use`, one per line at the top of a file, Kotlin's shape (owner, 2026-10-22): the name after `as` stands for another
# kit's path in this file, and `as` is always written. It names a module or a type too, so `use std::list as list;`
# brings back `list[i64]` and `list.append`. A rename onto a path, `as std::print`, is allowed (owner: "it's their
# choice"); every `use` gets a note in kayo's build report.
use std::list as list;
use std::print.shortest as say;
use std::print.stored as std::print;

!MAIN:
    variable $xs: list[integer.signed.64_bit] = [];
    for $i: integer.signed.64_bit [from: 1, to: 4, by: 1]:
        list.append(lend.mutable $xs, value: $i times $i);
    end of for
    say(list.length(lend $xs), " squares, the last ", $xs[position: 4], newline);
    std::print("0.1 as stored: ", 0.1, newline);
!end of MAIN
