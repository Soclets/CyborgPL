# A kit's build.cy is its build program, run by `kayo prepare`, not one of its modules
# (owner, 2026-10-09: build.cplr became build.cy); the oracle skips it.
!MAIN:
    std::print.shortest("built", newline);
!end of MAIN
