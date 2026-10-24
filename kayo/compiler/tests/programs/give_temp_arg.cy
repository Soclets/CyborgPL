# An owned value made only for a call is the function's from then on: the
# text from `to.stored.str.utf8[…]` or from another call was freed by the
# function and then again by the caller, and the program crashed in
# `free`. 3 + 5 + 2 = 10.
extern.C fn exit [$code: i32];
fn count [$s: str.utf8] -> i32 { return to.i32(std::str.bytes(lend $s)); }
fn make [$n: i32] -> str.utf8 { return to.stored.str.utf8($n); }
MAIN {
    let $a := count(to.stored.str.utf8(123));
    let $b := count(make(12345));
    let $c := count(make(-7));
    no_check my.exit($a plus $b plus $c);
}
