# Powers (owner, 2026-10-07): this kit lists `c_calls`, and the program denies every C call except
# `write`, so its one call to `write` builds. No std; CNC for C, and `vouch` for the `no_check`.

%external.C procedure write [$fd: integer.signed.32_bit, $buf: pointer[integer.unsigned.8_bit], $len: integer.unsigned.64_bit] -> integer.signed.64_bit;

!MAIN:
    let $text := "allowed
";
    !no_check let $n := write(1, lend $text, 8);
!end of MAIN
