# A keyword may name a function, reached by path, never bare (kits.md,
# 2026-09-19): `fn true` is legal and called as `my.true[]`.
extern.C fn exit [$code: i32];
# A keyword may name a function, reached by path, never bare (2026-09-19):
# bare `to` is the conversion and bare `true` the boolean, so these are `my.to[]`, `my.true[]`.
fn to [] -> i32 { return 5; }
fn true [] -> i32 { return 1; }
MAIN {
    let $x := my.to();
    let $y := my.true();
    no_check my.exit($x times 10 plus $y);
}
