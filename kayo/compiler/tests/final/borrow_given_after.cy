# The same give, after the borrow's last use: nothing still looks at 's'.
fn eat [let $s: str.utf8]:
;fn eat
fn first [lent $a: str.utf8] -> lent str.utf8:
    ret lend $a;
;fn first
fn len [lent $s: str.utf8] -> u64:
    ret std::str.bytes(lend $s);
;fn len
MAIN:
    let $s := "hello";
    let $l := first(lend $s);
    let $n := len(lend $l);
    eat(give $s);
;MAIN
