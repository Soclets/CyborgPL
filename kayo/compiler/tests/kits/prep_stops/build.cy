# Run by `cca prepare` in its own Kit's folder (owner, 2026-09-29): it
# says its name, and stops with 9 unless its folder's marker is beside it.
%extern.C proc write [$fd: i32, $buf: ptr[u8], $len: u64] -> i64;
%extern.C proc access [$path: ptr[u8], $mode: i32] -> i32;
%extern.C proc exit [$code: i32];
MAIN:
    let $at := "here-stops";
    no_check let $found := access(lend $at, 0);
    if $found != 0:
        no_check my.exit(9);
    ;if
    let $say := "stops";
    let $nl: std::ls[u8] := [10,];
    no_check let $w := write(1, lend $say, 5);
    no_check let $e := write(1, lend $nl, 1);
    no_check my.exit(0);
;MAIN
