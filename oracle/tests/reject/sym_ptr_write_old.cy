%extern.C proc malloc [let $bytes: int.unsign.64] -> ptr.mut[int.unsign.8];
!MAIN:
    !no_check let $p := malloc(4);
    !no_check change *$p = 65;
!end of MAIN
