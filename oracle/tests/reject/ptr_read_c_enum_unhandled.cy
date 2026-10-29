# ptr.read of a C enum from C can fail out_of_range (2026-09-28)
%layout.C enum Mode:
    emacs = 0;
;%layout.C enum Mode
%extern.C proc malloc [$bytes: u64] -> ptr.mut[Mode];
proc f [$p: ptr[Mode]] -> i32:
    no_check let $m := ptr.read($p);
    ret 0;
;proc f
MAIN:
    no_check let $mem := malloc(4);
    no_check let $x := f($mem);
;MAIN
