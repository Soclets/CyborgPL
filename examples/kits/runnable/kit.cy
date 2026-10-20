# A kit that is both a library and a program (owner, 2026-09-29: a): its MAIN runs when this kit is
# run itself, and is ignored when another kit uses it (examples/uses_a_kit).

bare.kit_prefix.never [fn];

pub fn seven [] -> i32 { return 7; }

MAIN {
    std::print.stored["runnable on its own", newln];
}
