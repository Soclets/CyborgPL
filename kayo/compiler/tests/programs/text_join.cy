%extern.C proc exit [$code: i32];
fn greet [lent $who: str.utf8] -> str.utf8:
    ret "hello, " concat $who;
;fn greet
MAIN:
    let $name: str.utf8 := "world";
    let $msg := greet(lend $name);
    let $n: i32 := std::str.bytes(lend $msg);
    let $plain: i32 := std::str.bytes(lend $name);
    no_check my.exit($n + $plain);
;MAIN
