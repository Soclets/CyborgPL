# In a `#strict` file a case says its value with `gives` (owner, 2026-09-29:
# b), and may do other work first (examples/strict_match.cpls as an exit
# code: "taken by bo" is 11 bytes).
extern.C fn exit [$code: i32];
enum Status { empty, claimed: str.utf8, locked }
fn label [$s: Status] -> str.utf8 {
    let immut $text := match $s {
        empty         { is "free"; }
        claimed $who { let immut $prefix := "taken by "; is $prefix concat $who; }
        locked        { is "locked"; }
    };
    return give $text;
}
fn width [$s: Status] -> i32 {
    let immut $l := label(give $s);
    let immut $n := std::str.bytes(lend $l);
    if $n > 1000 { return 0; }
    return to.i32($n);
}
MAIN {
    no_check my.exit(width(Status.claimed("bo")));
}
