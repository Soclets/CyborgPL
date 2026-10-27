# a function named `MAIN` is called by its path, never bare (owner, 2026-10-26)
function MAIN [] -> integer.signed.32_bit:
    return 1;
end of function MAIN
!MAIN:
    let $x := MAIN();
!end of MAIN
