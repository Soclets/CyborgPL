# `use`, one per line at the top of a file, Kotlin's shape (owner, 2026-10-22): the name after `as` stands for another
# kit's path in this file, and `as` is always written. It names a module or a type too, so `use std::list as list;`
# brings back `list[i64]` and `list.append`. A rename onto a path, `as std::print`, is allowed (owner: "it's their
# choice"); every `use` gets a note in kayo's build report.
use std::list as list;
use std::print.useful as say;
use std::print.stored as std::print;

MAIN {
    let mut 'xs': list[i64] = [];
    for 'i': i64 [from: 1, to: 4, by: 1] {
        list.append[lend.mut 'xs', value: 'i' x 'i'];
    }
    say[list.length[lend 'xs'], " squares, the last ", 'xs'[pos: 4], newln];
    std::print["0.1 as stored: ", 0.1, newln];
}
