# extern.C 'x': T — a C global (decided 2026-09-21). Reading or writing it is a no_check site.
%extern.C $lib_version: int.sign.32;              # C: extern int lib_version;
%extern.C var $lib_verbose: int.sign.32;          # C: extern int lib_verbose;  (we may write it)

!MAIN:
    !no_check let $v := $lib_version;
    std::print.shortest("version ", $v, newln);
    !no_check change $lib_verbose to 1;
    !no_check let $on := $lib_verbose;
    std::print.shortest("verbose ", $on, newln);
!end of MAIN
