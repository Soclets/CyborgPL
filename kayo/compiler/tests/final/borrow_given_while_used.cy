# A borrow lasts until its last use (Rust-style, core-features.md): giving
# 's' away while 'l' — a borrow of it that `first` handed back — is still
# used after is refused.
fn eat [$s: str.utf8] { }
fn first [lent $a: str.utf8] -> lent str.utf8 { return lend $a; }
fn len [lent $s: str.utf8] -> u64 { return std::str.bytes(lend $s); }
MAIN {
    let $s := "hello";
    let $l := first(lend $s);
    eat(give $s);
    let $n := len(lend $l);
}
