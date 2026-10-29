%extern.C proc exit [$code: i32];
fn inner [$n: i32] -> i32 or fail[bad: std::ls[i32]]:
    if $n < 0:
        fail bad[[1, 2, 3]];
    ;if
    ret $n;
;fn inner
fn outer [$n: i32] -> i32 or fail[bad: std::ls[i32]]:
    let $v := inner($n)?;
    ret $v + 1;
;fn outer
MAIN:
    let $ok := outer(5) ?? 0;
    var $n: i32 = 0;
    outer(-1)-
    otherwise:
        case bad $why:
            change $n to std::ls.len(lend $why);
        ;case bad
    ;otherwise;
    no_check my.exit($ok + $n);
;MAIN
