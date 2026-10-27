# Handing back a borrow (owner, 2026-09-29): `-> lent @x T`. `@x` names a lifetime group, announced
# after the function's name (owner, 2026-10-02): everything marked `@x` lives and dies together, so the result of `longer`
# is good while both 'a and 'b are; `pick`'s result only needs 'a. Like Rust's `'x`, spelt with
# `@` since `'` means a variable here. A borrow of something that dies at the return is refused.
# The group may be left out (`first`), and the prover works it out from the body (a `#strict` file
# had to write it, until `#strict` went on 2026-10-07).
fn longer [@x] [lent @x $a: str.utf8, lent @x $b: str.utf8] -> lent @x str.utf8:
    if std::str.bytes(lend $a) at_least std::str.bytes(lend $b):
        ret lend $a;
    end of if
    ret lend $b;
end of fn longer
fn pick [@x, @y] [lent @x $a: str.utf8, lent @y $b: str.utf8] -> lent @x str.utf8:
    ret lend $a;
end of fn pick
fn first [lent $a: str.utf8] -> lent str.utf8:
    ret lend $a;
end of fn first
!MAIN:
    let $s := "hello";
    let $t := "hi";
    let $l := longer(lend $s, lend $t);
    let $p := pick(lend $t, lend $s);
    let $f := first(lend $s);
    std::print.shortest($l, " ", $p, " ", $f, newln);
!end of MAIN
