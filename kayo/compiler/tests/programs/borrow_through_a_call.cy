# A function hands back the borrow another call handed it (check claude,
# 2026-09-29, 46faa74): `second`'s result borrows its own lent 'a', through
# `first`. The oracle prints 2 for the same program.
extern.C fn exit [$code: i32];
fn first [lent $a: std::list[i32]] -> lent std::list[i32] { return lend $a; }
fn second [lent $a: std::list[i32]] -> lent std::list[i32] { return first(lend $a); }
MAIN {
    let $s: std::list[i32] = [1, 2, 3];
    let $r := second(lend $s);
    no_check my.exit($r[pos: 2]);
}
