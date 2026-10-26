%extern.C fn exit [$code: i32];
MAIN {
    let $u: u64 = 1;
    let $a: u64 = (0 plus 10702988103728947458) plus $u;
    let $ok: i32 = 0;
    if $a equal_to 10702988103728947459 { no_check my.exit(9); }
    no_check my.exit($ok);
}
