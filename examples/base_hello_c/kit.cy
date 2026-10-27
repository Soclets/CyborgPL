# Hello World through C (design/cores.md, "Hello World at every level"): no std, only CNC, which C
# calls need. The call is a `no_check` site, marked: the marker is the base's since IEG was
# dissolved (owner, 2026-10-12), so it is written everywhere, as the `vouch` power is listed.

%external.C procedure write [$fd: integer.signed.32_bit, $buf: pointer[integer.unsigned.8_bit], $len: integer.unsigned.64_bit] -> integer.signed.64_bit;

!MAIN:
    let $text := "Hello, world!
";
    !no_check let $n := write(1, lend $text, 14);
!end of MAIN
