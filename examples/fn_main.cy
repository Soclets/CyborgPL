# A function may be named `MAIN`, as a keyword may (owner, 2026-10-26: "yes"), reached only by its path: bare `MAIN`
# is always the program's block.
fn MAIN [] -> str.utf8:
    ret "from the function MAIN";
end of fn MAIN

!MAIN:
    std::print.shortest(my.MAIN(), newln);
!end of MAIN
