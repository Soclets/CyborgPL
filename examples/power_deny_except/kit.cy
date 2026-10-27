# Powers (owner, 2026-10-07): this kit lists `c_calls`, and the program denies every C call except
# `write`, so its one call to `write` builds. No std; CNC for C, and `vouch` for the `no_check`.

%extern.C proc write [$fd: int.sign.32, $buf: ptr[int.unsign.8], $len: int.unsign.64] -> int.sign.64;

!MAIN:
    let $text := "allowed
";
    !no_check let $n := write(1, lend $text, 8);
!end of MAIN
