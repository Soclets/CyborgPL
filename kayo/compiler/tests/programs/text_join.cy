%extern.C fn exit [$code: i32];
fn greet [lent $who: str.utf8] -> str.utf8 {
    return "hello, " concat $who;
}
MAIN {
    let $name: str.utf8 = "world";
    let $msg := greet(lend $name);
    let $n: i32 = std::str.bytes(lend $msg);
    let $plain: i32 = std::str.bytes(lend $name);
    no_check my.exit($n plus $plain);
}
