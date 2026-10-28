# A header in the kit (owner, 2026-10-10): `incl.C["…"]`, from the kit's root. Its plain
# `#define`s are consts (C's `0644` is octal, so 420); `MYLIB_MAX`, a macro that looks like a function,
# is skipped; `mylib_add` is declared, as `extern.C fn mylib_add [$p1: i32, $p2: i32] -> i32;`
# would be, though the oracle cannot run it (kayo links the library).
incl.%C["vendor/mylib.h"];

MAIN:
    std::println($MYLIB_NAME, " ", $MYLIB_VERSION, " ", $MYLIB_FLAGS);
;MAIN
