# A function hands back a borrow (owner, 2026-09-29): `-> lent@x T`, where
# `@x` names a lifetime group; left out, as in `first`, the body says what
# the result depends on (examples/lifetimes.cpls as an exit code: "hello",
# "hi" and "hello" are 5, 2 and 5 bytes → 5 x 100 + 2 x 10 + 5 = 525,
# which exits as 525 mod 256 = 13).
%extern.C proc exit [let $code: i32];
fn longer [@x] [lent @x $a: str.utf8, lent @x $b: str.utf8] -> lent @x str.utf8:
    if std::str.bytes(lend $a) >= std::str.bytes(lend $b):
        ret lend $a;
    ;if
    ret lend $b;
;fn longer
fn pick [@x, @y] [lent @x $a: str.utf8, lent @y $b: str.utf8] -> lent @x str.utf8:
    ret lend $a;
;fn pick
fn first [lent $a: str.utf8] -> lent str.utf8:
    ret lend $a;
;fn first
fn width [lent $s: str.utf8] -> i32:
    let $n := std::str.bytes(lend $s);
    if $n > 1000:
        ret 0;
    ;if
    ret to.i32($n);
;fn width
fn run [] -> i32:
    let $s := "hello";
    let $t := "hi";
    let $l := longer(lend $s, lend $t);
    let $p := pick(lend $t, lend $s);
    let $f := first(lend $s);
    ret width(lend $l) x 100 + width(lend $p) x 10 + width(lend $f);
;fn run
MAIN:
    no_check my.exit(run());
;MAIN
