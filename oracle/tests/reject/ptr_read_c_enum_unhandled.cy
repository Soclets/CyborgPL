# ptr.read of a C enum from C can fail out_of_range (2026-09-28)
%layout.C enumeration Mode:
    emacs = 0;
end of enumeration Mode
%external.C procedure malloc [$bytes: integer.unsigned.64_bit] -> pointer.mutable[Mode];
procedure f [$p: pointer[Mode]] -> integer.signed.32_bit:
    !no_check let $m := pointer.read($p);
    return 0;
end of procedure f
!MAIN:
    !no_check let $mem := malloc(4);
    !no_check let $x := f($mem);
!end of MAIN
