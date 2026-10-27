# extern.C 'x': T — a C global (decided 2026-09-21). Reading or writing it is a no_check site.
%external.C $lib_version: integer.signed.32_bit;              # C: extern int lib_version;
%external.C variable $lib_verbose: integer.signed.32_bit;          # C: extern int lib_verbose;  (we may write it)

!MAIN:
    !no_check let $v := $lib_version;
    std::print.shortest("version ", $v, newline);
    !no_check change $lib_verbose = 1;
    !no_check let $on := $lib_verbose;
    std::print.shortest("verbose ", $on, newline);
!end of MAIN
