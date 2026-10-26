# A function may be named `MAIN`, as a keyword may (owner, 2026-10-26: "yes"), reached only by its path: bare `MAIN`
# is always the program's block.
fn MAIN [] -> str.utf8 { return "from the function MAIN"; }

MAIN {
    std::print.useful(my.MAIN(), newln);
}
