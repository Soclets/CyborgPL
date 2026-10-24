MAIN {
    let $limit := 60;
    let $f := fn [$s: i32] -> bool { return $s at_least $limit; };
    let $b := $f(70);
}
