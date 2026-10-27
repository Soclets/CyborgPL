# A function may be named `MAIN`, as a keyword may (owner, 2026-10-26: "yes"), reached only by its path: bare `MAIN`
# is always the program's block.
function MAIN [] -> string.utf8:
    return "from the function MAIN";
end of function MAIN

!MAIN:
    std::print.shortest(my.MAIN(), newline);
!end of MAIN
