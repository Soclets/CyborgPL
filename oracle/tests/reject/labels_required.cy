coll.fn say {
    labels [$t as value];
    say [$n: i32, $t: i32] -> i32 { return $t; }
}
MAIN { let $x := say(1); }
