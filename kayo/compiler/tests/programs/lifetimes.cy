# A function hands back a borrow (owner, 2026-09-29): `-> lent@x T`, where
# `@x` names a lifetime group; left out, as in `first`, the body says what
# the result depends on (examples/lifetimes.cpls as an exit code: "hello",
# "hi" and "hello" are 5, 2 and 5 bytes → 5 x 100 + 2 x 10 + 5 = 525,
# which exits as 525 mod 256 = 13).
extern.C fn exit [$code: i32];
fn longer [@x] [lent @x $a: str.utf8, lent @x $b: str.utf8] -> lent @x str.utf8 {
    if std::str.bytes(lend $a) at_least std::str.bytes(lend $b) { return lend $a; }
    return lend $b;
}
fn pick [@x, @y] [lent @x $a: str.utf8, lent @y $b: str.utf8] -> lent @x str.utf8 {
    return lend $a;
}
fn first [lent $a: str.utf8] -> lent str.utf8 {
    return lend $a;
}
fn width [lent $s: str.utf8] -> i32 {
    let $n := std::str.bytes(lend $s);
    if $n > 1000 { return 0; }
    return to.i32($n);
}
fn run [] -> i32 {
    let $s := "hello";
    let $t := "hi";
    let $l := longer(lend $s, lend $t);
    let $p := pick(lend $t, lend $s);
    let $f := first(lend $s);
    return width(lend $l) times 100 plus width(lend $p) times 10 plus width(lend $f);
}
MAIN {
    no_check my.exit(run());
}
