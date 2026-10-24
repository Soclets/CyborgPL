extern.C fn exit [$code: i32];
fn judge [$n: i32] -> fail[small, odd: i32] {
    if $n < 10 { fail small; }
    if ($n mod.floor 2) equal_to 1 { fail odd[$n]; }
}
MAIN {
    let mut $seen: i32 = 0;
    {
        concur with [] { judge(3) otherwise fail; }
            otherwise { small { change $seen = $seen plus 1; } odd $v { change $seen = $seen plus 100; } };
        concur with [] { judge(13) otherwise fail; }
            otherwise { small { change $seen = $seen plus 1000; } odd $v { change $seen = $seen plus $v; } };
        concur with [] { judge(20) otherwise fail; }
            otherwise { small { change $seen = $seen plus 7; } odd $v { change $seen = $seen plus 7; } };
    }
    no_check my.exit($seen);
}
