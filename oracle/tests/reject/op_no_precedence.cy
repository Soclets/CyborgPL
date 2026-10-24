struct V { x: i32 }
op <+> [$a: V, $b: V] -> V { return V[x = $a.x plus $b.x]; }
MAIN {
    let $u := V[x = 1];
    let $w := $u <+> $u <+> $u;
}
