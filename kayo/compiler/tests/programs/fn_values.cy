# Functions as values (functions-as-values.md): a named function by its
# bare name, held in a variable and called as 'f'[…], passed to a function
# that takes a `fn[…]`; a literal capturing nothing, and one borrowing with
# `with [lend …]`, called from inside a loop. Values chosen at run time.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
rec Locker:
    var number: i32; var owner: str.utf8;
;rec Locker
fn by_number [lent $a: Locker, lent $b: Locker] -> bool:
    ret $a.number < $b.number;
;fn by_number
fn by_owner [lent $a: Locker, lent $b: Locker] -> bool:
    ret std::str.bytes(lend $a.owner) < std::str.bytes(lend $b.owner);
;fn by_owner
fn first_by [lent $xs: std::ls[Locker], let $less: fn[lent Locker, lent Locker] -> bool] -> i32:
    var $best = 1;
    for $i [from: 2, to: std::ls.len(lend $xs), by: 1]:
        if $less(lend $xs[pos: $i], lend $xs[pos: $best]):
            change $best to $i;
        ;if
    ;for
    let $winner := lend $xs[pos: $best];
    ret $winner.number;
;fn first_by
fn count_if [lent $xs: std::ls[i32], let $test: fn[i32] -> bool] -> i32:
    var $n: i32 = 0;
    for $x in lend $xs:
        if $test[pos: $x]:
            change $n to $n + 1;
        ;if
    ;for
    ret $n;
;fn count_if
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $got: i32 = 0;
    var $first: i32 = 9;
    if $there:
        change $first to 7;
    ;if
    let $lockers := [Locker[number = $first, owner = "Cy"], Locker[number = 2, owner = "Annabel"], Locker[number = 5, owner = "Bo"]];
    if first_by(lend $lockers, by_number) == 2:
        change $got to $got + 1;
    ;if
    if first_by(lend $lockers, by_owner) == $first:
        change $got to $got + 2;
    ;if
    let $f := by_number;
    if $f(lend $lockers[pos: 2], lend $lockers[pos: 1]):
        change $got to $got + 4;
    ;if
    var $limit = 60;
    if $there:
        change $limit to 50;
    ;if
    let $scores := [72, 40, 95, 55];
    if count_if(lend $scores, fn [let $s: i32] -> bool with [lend $limit]:
        ret $s >= $limit;
    ;fn) == 3:
        change $got to $got + 8;
    ;)
    if count_if(lend $scores, fn [let $s: i32] -> bool:
        ret ($s mod.floor 2) == 0;
    ;fn) == 2:
        change $got to $got + 16;
    ;)
    let $g := fn [let $s: i32] -> bool with [lend $limit]:
        ret $s < $limit;
    ;fn;
    var $h = $g;
    if $there:
        change $h to fn [let $s: i32] -> bool:
            ret $s > 90;
        ;fn;
    ;if
    if count_if(lend $scores, $h) == 1 and count_if(lend $scores, $g) == 1:
        change $got to $got + 32;
    ;if
    no_check my.exit($got);
;MAIN
