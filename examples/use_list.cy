# Several `use` renames in one line, in `[ ]`, each with its `as` (owner, 2026-10-24: "If use doesn't support doing a lot of
# shit in one line, it should", then "Yes"); one alone may go bare or in brackets.
use [std::print.shortest as printu, std::list as list, std::string.join as join];
!MAIN:
    let $xs: list[string.utf8] = ["a", "b"];
    printu(join(lend $xs, between: "+"), newline);
!end of MAIN
