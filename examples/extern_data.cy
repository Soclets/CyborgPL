# extern.C 'x': T — a C global (decided 2026-09-21). Reading or writing it is a no_check site.
extern.C $lib_version: i32;              # C: extern int lib_version;
extern.C mut $lib_verbose: i32;          # C: extern int lib_verbose;  (we may write it)

MAIN {
    no_check let $v := $lib_version;
    std::print.useful("version ", $v, newln);
    no_check change $lib_verbose = 1;
    no_check let $on := $lib_verbose;
    std::print.useful("verbose ", $on, newln);
}
