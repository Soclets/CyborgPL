coll.fn say {
    labels [$t as value];
    say [$t: i32] -> i32 { return $t; }
}
MAIN { let $x := say(colour: 1); }
