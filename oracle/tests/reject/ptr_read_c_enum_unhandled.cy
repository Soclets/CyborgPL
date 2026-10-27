# ptr.read of a C enum from C can fail out_of_range (2026-09-28)
%layout.C enum Mode:
    emacs = 0;
end of enum Mode
%extern.C proc malloc [let $bytes: int.unsign.64] -> ptr.mut[Mode];
proc f [let $p: ptr[Mode]] -> int.sign.32:
    !no_check let $m := ptr.read($p);
    ret 0;
end of proc f
!MAIN:
    !no_check let $mem := malloc(4);
    !no_check let $x := f($mem);
!end of MAIN
