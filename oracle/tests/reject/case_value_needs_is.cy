enum S { a, b }
MAIN {
    let $x := match S.a { a { 1 } b { 2 } };
}
