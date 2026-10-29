# Functions as values (design/functions-as-values.md, 2026-09-21).
rec Locker:
    var number: i32;
    var owner: str.utf8;
;rec Locker

fn by_number [lent $a: Locker, lent $b: Locker] -> bool:
    ret $a.number < $b.number;
;fn by_number
fn by_owner  [lent $a: Locker, lent $b: Locker] -> bool:
    ret std::str.bytes(lend $a.owner) < std::str.bytes(lend $b.owner);
;fn by_owner

# a function that takes a function: the type is the signature without the names
fn first_by [lent $xs: std::ls[Locker], $less: fn[lent Locker, lent Locker] -> bool] -> str.utf8:
    var $best = 1;
    for $i [from: 2, to: std::ls.len(lend $xs), by: 1]:
        if $less(lend $xs[pos: $i], lend $xs[pos: $best]):
            change $best to $i;
        ;if
    ;for
    let $winner := lend $xs[pos: $best];
    ret copy $winner.owner;
;fn first_by

fn keep [lent $xs: std::ls[i32], $test: fn[i32] -> bool] -> std::ls[i32]:
    var $out: std::ls[i32] = [];
    for $x in lend $xs:
        if $test($x):
            std::ls.append(lend.mut $out, value: $x);
        ;if
    ;for
    ret give $out;
;fn keep

MAIN:
    let $lockers := [Locker[number = 7, owner = "Cy"], Locker[number = 2, owner = "Ann"], Locker[number = 5, owner = "Bo"]];
    std::print.stored(first_by(lend $lockers, by_number), newln);          # a named function, as a value: its bare name
    std::print.stored(first_by(lend $lockers, by_owner), newln);

    let $f := by_number;                                                 # held in a variable
    std::print($f(lend $lockers[pos: 1], lend $lockers[pos: 2]), newln);   # called: 'f'[…]

    let $limit := 60;
    let $scores := [72, 40, 95, 55];
    let $passing := keep(lend $scores, fn [$s: i32] -> bool with [lend $limit]:
        ret $s >= $limit;
    ;fn);
    std::print($passing, newln);

    let $evens := keep(lend $scores, fn [$s: i32] -> bool:
        ret ($s mod.floor 2) == 0;
    ;fn);   # captures nothing
    std::print($evens, newln);
;MAIN
