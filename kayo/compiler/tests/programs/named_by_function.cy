# A template's name worked out by a function of the file, run while
# generating (owner, 2026-10-13: `named size_name($n);`): kb, mb and pow6.
# 1 + 1 + 64 is 66.
%extern.C fn exit [$code: i32];

fn size_name [$n: i32] -> str.utf8 {
    if $n equal_to 1024 { return "kb"; }
    if $n equal_to 1048576 { return "mb"; }
    let mut $k := 0;
    let mut $m := $n;
    while $m > 1 {
        change $m = $m div.floor 2;
        change $k = $k plus 1;
    }
    return "pow" concat to.stored.str.utf8($k);
}

coll.fn block {
    for $n in [1024, 1048576, 64] {
        named size_name($n);
        [] -> i32 { return $n; }
    }
    #generated_3_lines_below
    block.kb [] -> i32 { return 1024; }
    block.mb [] -> i32 { return 1048576; }
    block.pow6 [] -> i32 { return 64; }
}

MAIN {
    let $t := block.kb() div.floor 1024 plus block.mb() div.floor 1048576 plus block.pow6();
    no_check my.exit(to.i32($t));
}
