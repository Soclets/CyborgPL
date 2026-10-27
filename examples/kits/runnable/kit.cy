# A kit that is both a library and a program (owner, 2026-09-29: a): its MAIN runs when this kit is
# run itself, and is ignored when another kit uses it (examples/uses_a_kit).


public function seven [] -> integer.signed.32_bit:
    return 7;
end of function seven

!MAIN:
    std::print.stored("runnable on its own", newline);
!end of MAIN
