# Handing back a borrow (owner, 2026-09-29): `-> lent @x T`. `@x` names a lifetime group, announced
# after the function's name (owner, 2026-10-02): everything marked `@x` lives and dies together, so the result of `longer`
# is good while both 'a and 'b are; `pick`'s result only needs 'a. Like Rust's `'x`, spelt with
# `@` since `'` means a variable here. A borrow of something that dies at the return is refused.
# The group may be left out (`first`), and the prover works it out from the body (a `#strict` file
# had to write it, until `#strict` went on 2026-10-07).
function longer [@x] [lent @x $a: string.utf8, lent @x $b: string.utf8] -> lent @x string.utf8:
    if std::string.bytes(lend $a) at_least std::string.bytes(lend $b):
        return lend $a;
    end of if
    return lend $b;
end of function longer
function pick [@x, @y] [lent @x $a: string.utf8, lent @y $b: string.utf8] -> lent @x string.utf8:
    return lend $a;
end of function pick
function first [lent $a: string.utf8] -> lent string.utf8:
    return lend $a;
end of function first
!MAIN:
    let $s := "hello";
    let $t := "hi";
    let $l := longer(lend $s, lend $t);
    let $p := pick(lend $t, lend $s);
    let $f := first(lend $s);
    std::print.shortest($l, " ", $p, " ", $f, newline);
!end of MAIN
