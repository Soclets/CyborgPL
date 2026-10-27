# a function named `MAIN` is called by its path, never bare (owner, 2026-10-26)
fn MAIN [] -> int.sign.32:
    ret 1;
end of fn MAIN
!MAIN:
    let $x := MAIN();
!end of MAIN
