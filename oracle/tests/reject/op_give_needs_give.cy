struct V { x: i32 }
op <*> [give $a: V, $b: i32] -> V { return give $a; }
MAIN {
    let $u := V[x = 1];
    let $w := $u <*> 2;
}
