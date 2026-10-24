layout.C union Number { i: i32, f: f32 }

MAIN {
    let $n := Number[i = 1];
    let $x := $n.i;
}
