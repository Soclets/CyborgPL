# Collections (owner, 2026-10-13): a preset fills some of the main
# function's arguments, passes the rest as the main function takes them
# (`lend` for the lent list) and fails as it fails; a member may point at a
# function written elsewhere (owner, 2026-10-19), in a collection with its
# own function and in one without. 73 + 12 + 100 (the empty list's
# failure) + 7 is 192, + 9 (`mix.square[3]`) + 4 (`only.twice[2]`) is 205.
%extern.C proc exit [let $code: i32];

fn square [let $x: i32] -> i32:
    ret $x x $x;
;fn square
fn double [let $x: i32] -> i32:
    ret $x x 2;
;fn double

coll.fn mix:
    member mix [lent $xs: std::ls[i32], let $a: i32, let $b: i32] -> i32 or fail[empty]:
        if std::ls.len(lend $xs) == 0:
            fail empty;
        ;if
        ret $a x 10 + $b;
    ;member mix
    member mix.seven [$a = 7];
    member mix.both [$a = 1, $b = 2];
    member mix.square = square;
    for $n [from: 1, to: 2, by: 1]:
        named "plus_" concat to.stored.str.utf8($n);
        member [let $x: i32] -> i32:
            ret $x + $n;
        ;member
    ;for
    #generated_6_lines_below
    member mix.plus_1 [let $x: i32] -> i32:
        ret $x + 1;
    ;member mix.plus_1
    member mix.plus_2 [let $x: i32] -> i32:
        ret $x + 2;
    ;member mix.plus_2
;coll.fn mix

coll.fn only:
    member only.twice = double;
;coll.fn only

MAIN:
    let $xs: std::ls[i32] := [1, 2];
    let $none: std::ls[i32] := std::ls.empty;
    let $a := mix.seven(lend $xs, 3) ?? 0;
    let $b := mix.both(lend $xs) ?? 0;
    let $c := mix.both(lend $none) ?? 100;
    let $t := $a + $b + $c + mix.plus_2(5) + mix.square(3) + only.twice(2);
    no_check my.exit(to.i32($t));
;MAIN
