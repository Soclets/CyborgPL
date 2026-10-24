enum S { a, b }
MAIN {
    let $s := S.a;
    match $s { a { } b { } z { } }
}
