# `list.remove` checks its position, as a read does (check claude,
# 2026-10-08: past the end it read freed memory and died of SIGBUS). The
# position here is the process ID, unknown until it runs: the check fires.
extern.C fn getpid [] -> int32;
extern.C fn exit ['code': int32];
MAIN {
    let mut 'xs': list[int32] = [1, 2, 3];
    no_check let 'n': int32 = getpid[];
    let 'v': int32 = list.remove[lend.mut 'xs', pos 'n'];
    no_check my.exit['v'];
}
