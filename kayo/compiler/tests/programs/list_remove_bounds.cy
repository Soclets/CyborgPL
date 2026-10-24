# `list.remove` checks its position, as a read does (check claude,
# 2026-10-08: past the end it read freed memory and died of SIGBUS). The
# position here is the process ID, unknown until it runs: the check fires.
extern.C fn getpid [] -> i32;
extern.C fn exit [$code: i32];
MAIN {
    let mut $xs: std::list[i32] = [1, 2, 3];
    no_check let $n: i32 = getpid();
    let $v: i32 = std::list.remove(lend.mut $xs, pos: $n);
    no_check my.exit($v);
}
