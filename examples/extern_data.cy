# A C global (decided 2026-09-21), saying whether it can change (owner, 2026-10-27): `let` for one C declares `const`,
# `var` for the rest. Reading or writing it is a no_check site.
%extern.C let $lib_version: i32;          # C: extern const int lib_version;
%extern.C var $lib_verbose: i32;          # C: extern int lib_verbose;  (we may write it)

MAIN:
    no_check let $v := $lib_version;
    std::println("version ", $v);
    no_check change $lib_verbose to 1;
    no_check let $on := $lib_verbose;
    std::println("verbose ", $on);
;MAIN
