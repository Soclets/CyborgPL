# Hello World through C (design/cores.md, "Hello World at every level"): no std, only CNC, which C
# calls need. The call is a `no_check` site, marked: the marker is the base's since IEG was
# dissolved (owner, 2026-10-12), so it is written everywhere, as the `vouch` power is listed.

%extern.C proc write [let $fd: int.sign.32, let $buf: ptr[int.unsign.8], let $len: int.unsign.64] -> int.sign.64;

!MAIN:
    let $text := "Hello, world!
";
    !no_check let $n := write(1, lend $text, 14);
!end of MAIN
