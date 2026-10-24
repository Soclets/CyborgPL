op <+> [$a: i32, $b: i32] -> i32 { return $a plus $b; }
MAIN {
    let $b := 2;
    let $w := 1 <+>-$b;
}
