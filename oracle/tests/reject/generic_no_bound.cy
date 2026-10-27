# a generic says what it accepts (owner, 2026-10-02: "Don't default to allow all")
fn f <T> [let $a: <T>] -> <T>:
    ret $a;
end of fn f
!MAIN:
!end of MAIN
