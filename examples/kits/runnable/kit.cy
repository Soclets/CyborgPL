# A kit that is both a library and a program (owner, 2026-09-29: a): its MAIN runs when this kit is
# run itself, and is ignored when another kit uses it (examples/uses_a_kit).


pub fn seven [] -> int.sign.32:
    ret 7;
end of fn seven

!MAIN:
    std::print.stored("runnable on its own", newln);
!end of MAIN
