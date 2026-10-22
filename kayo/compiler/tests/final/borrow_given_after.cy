# The same give, after the borrow's last use: nothing still looks at 's'.
fn eat ['s': str.utf8] { }
fn first [lent 'a': str.utf8] -> lent str.utf8 { return lend 'a'; }
fn len [lent 's': str.utf8] -> u64 { return std::str.bytes[lend 's']; }
MAIN {
    let 's' := "hello";
    let 'l' := first[lend 's'];
    let 'n' := len[lend 'l'];
    eat[give 's'];
}
